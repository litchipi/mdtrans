I currently follow the awesome [rust-raspberrypi-OS tutorial][raspi_rust_tuto] to learn how to
do some bare-metal programming on my Raspberry Pi 3.

> The code for the examples are taken from this tutorial

It uses the macros `register_bitfield!` and `register_structs!`, similar to the [bitflags! macro](https://docs.rs/bitflags/latest/bitflags/),
that I saw in many places in embedded Rust codes.

However I wasn't able to read it easily and grasp what it was doing, and felt like digging deeper.  
That's when [@devonmorris](https://hachyderm.io/@devonmorris) proposed to use `cargo-expand` to look the code after the macro is resolved.

This post will explain what registers are, what are their use in bare-metal programming, and how these macros allows for smooth Rust code when working with them.

Throughout this post, we'll use the [datasheet for the CPU of the Raspberry Pi 3][rpi3_cpu_datasheet] as an example, but this principle applies to any CPU,
just grab its datasheet and read it up !

# Introduction

> If you already worked with embedded systems, you can probably skip this section.  
If you don't skip it, please check it for mistakes I may have written :-)

When working with embedded systems, or low level programming (in the Linux kernel for example), we make extensive use of memory addresses to store things.

It may be an area of the RAM to store a variable, or a way to communicate with the GPU (using a *mailbox*),
or even a peripheral thanks to the Memory-Mapped I/O (*MMIO*) embedded inside the CPU.

This way, a program running bare-metal on a Raspberry Pi for example could lit the onboard LED just by writing the right value at the right memory address.

How do we find where to do this ?  
Just get the datasheet, and look up the peripheral you want to interact with ! (for example, the GPIO section is on **page 90**)

> Note that on the Raspberry pi *board*, the GPIO numbers may not be the same as the CPU. We are inside the CPU, not on the board.
 
> To check what board device is connected to what CPU GPIO, you can look inside the [device-tree file][rpi_dt] and look up your board version, the "pin@N" on the left refers to the CPU pin

What you have here, is a list of registers, their memory address, their size,
and the register to write to if we want to set a GPIO to High or Low (**GPSETn** and **GPCLEARn**).

To set a GPIO to High (as we see in **page 95**), we set a *single bit* of the 32-bit value stored
at this memory address (what is called a **register**).

The register `GPSET0` can be viewed as a "field of bits", where to access some of the bits of this
field, you have to check a portion of the binary data of the whole value.

Let's treat it this way, as a `struct` (or `class` if you're Pythonic),
containing fields we can *get* or *set*.

## Set the value of a register

As we can see on the datasheet, to **set** the GPIO number **5** of the CPU, you need to write `[..20 more zeros..]_0000_0001_0000` (or `0x20`, or `32`)
to the memory address of `GPSET0` which is (on page 90) `0x7E20_001C`.

As you can see on the binary value, to set the pin `5`, we have a `1` and `4` zeros after,
it's like **shifting 1 to the left 4 times**, this operation is generally noted `1 << 4`
in programming languages.

So to set the pin `N`, we write `1 << (N - 1)` to the address of the register `GPSETn`  
(as stated in the datasheet, if `N < 32` we use `GPSET0`, if `32 < N < 54` we use `GPSET1`)

> Note that this way we can set multiple pins at the same time, using an **AND** operation (usually noted `|`), we can set multiple bits like so: `(1 << 4) | (1 << 12) | (1 << 8)`

## Get the value of a register

If a register holds the value `[...]_1101_1001`, how do we check if the 5th and 6th bit are set ?

For this, we use a `mask`, a value that will only leave the bits we are interested in if
we apply it on the value of our register with an **AND** (`&`).

For example:  
`1101_1001 & 0110_000 (mask) = 0100_000`  
The mask allowed us to only leave the 2 bits we were interested in.

To check its value, we can shift it to the right `(0b0100_0000 >> 4) = 0b0000_0010`, giving us the bits we are interested in, `0b10`.

> Note that for a single bit, we can just check that the value is > 0 to see if the bit is set, `0b0100_0000 > 0 = true`, `0b0000_0000 > 0 = false`.

## In the real world

Usually, the manufacturer of CPUs meant to be programmed in bare-metal will provide some `hal` (Hardware Abstraction Layer),
a library containing all the memory addresses for the registers.

If working on a common board, it may even exist a `bsp` (Board Support Package), that contains a lot of useful tools, functions and constants to play with the hardware.

When we deal with memory addresses directly, we are basically re-creating the `hal`

# Rusting it up

We can of course do everything from scratch in Rust, get the memory addresses and such, but the crate [tock_registers][tock_registers_crate] allows us to get a much better API using macros !

``` rust
register_bitfields!{
    u32,
    GPFSEL1 [
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ],
    ],
}
```

This macro defines a list of registers that holds a `u32` (32 bits), containing a single register `GPFSEL1`.

> In this example, this register allows to select wether we want a pin to be treated as an *Input* or *Output* (or a special function).  
The **FSEL15** field of this register refers to the configuration for the *pin number 15*

In this register `GPFSEL1` of 32 bits, the value of `3 bits` starting at the offset `15` is named `FSEL15`, and can hold 3 different values:
- `000`, configuration as an **Input**
- `001`, configuration as an **Output**
- `100`, configuration as its **Alternate Function 0**, which in this case is the **UART RX**

This sets up the memory addresses, fields and values, but not the struct interfaces yet, we need to use another macro for this:

``` rust
register_structs! {
    RegisterBlock {
        (0x00 => _unused1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _unused2),
        (0x94 => @END),
    }
}
```

For now, it seams a bit mysterious, but we'll expand that macro later and understand how it works.

What it does, is that it creates a struct `RegisterBlock` that holds a list of registers,
some of them are not even defined (`_unused`),
and at the offset `0x04` we find our previously defined register `GPFSEL1`
that we configure with the `ReadWrite` permissions.

Now, this allows that kind of code:

``` rust
let registers : RegisterBlock = // We'll see how it's initialized later ...
registers.GPFSEL1.write(GPFSEL1::FSEL15::Input)   // We erase all the register, and set FSEL15 to Input
registers.GPFSEL1.modify(GPFSEL1::FSEL14::Output) // We keep the changes made before, and set FSEL14 to Output
registers.GPFSEL1.write(GPFSEL1::CLEAR)           // We erase all the data in the register
```

> The `RegisterBlock` refers to all the registers that we want to put together in our struct.  
In our case we can define one made for interfacing with the GPIO.  

## Inspecting the macro

But what does it do exactly ? How does it work ?

I'll use [`cargo-expand`][cargo_expand] here, and will simplify the code so we can have a clear view on it

### Register bitfields

To start, I'll inspect the macro `register_bitfield` we used before

``` rust
register_bitfields!{
    u32,
    GPFSEL1 [
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ],
    ],
}
```

First, it generates a module that will hold all the defined structs, enums and constants.

Inside, a `Register` struct is defined, which will be useful for generic type definition on the `Field` type, which will be our interface.

In itself, the struct doesn't do much, but uses the type system to later set the generic types to a type related to `GPFSEL1` only.

This allows the code to use the form `GPFSEL1::Register`

``` rust
mod GPFSEL1 {
    #[derive(Clone, Copy)]
    pub struct Register;

    // [ ..snip.. ]
}
```

Then, a constant `FSEL15` is created, of type `Field`, which is an interface for the whole register value.

The arguments passed to its creation function is the `mask` and the `offset` used.  
We defined `FSEL15` to be a **3 bit wide** field, so the mask will be `[...]_0000_0111`,
as it has an **offset of 15**, the mask will be shifted to the left of `15` rows in order to get the final mask.

To get the basic mask, we perform the computation:
- `A = 1 << 2`, which gives `A = [...]_0000_0100`
- `B = (1 << 2) - 1`, which gives `B = [...]_0000_0011`
- `mask = A + B`, resulting to `mask = [...]_0000_0111`

``` rust
mod GPFSEL1 {
    // [ ..snip.. ]
    // Eye candied
    let mask = (1 << (3 - 1)) + ((1 << (3 - 1)) - 1);
    pub const FSEL15 = Field::new(mask, 15);
    // [ ..snip.. ]
}
```

The macro then generates a new sub-module `FSEL15` which will use the `FieldValue` type to define the interfaces to this field.

As we gave the possible values for this field inside the macro, it generates 2 possible interfaces:
- An enum `Value`, which can be translated to `FieldValue` to be applied, and can be used as `u32` directly
- A constant for each possible value, of type `FieldValue`

In addition, it will generate the `SET` constant that holds the `FieldValue` linked to the max value (all bits set to `1`),
and a `CLEAR` constant for all bits set to `0`.
    
This interface allows to write `GPFSEL1::FSEL15::Value::Input`, or directly `GPFSEL1::FSEL15::Input`.

``` rust
mod GPFSEL1 {
    // [ ..snip.. ]
    pub mod FSEL15 {
        use super::Register;

        #[repr(u32)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum Value {
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100,
        }

        // In reality, the computation is put directly in the code
        // This is only for your pretty eyes
        let mask = (1 << (3 - 1)) + ((1 << (3 - 1)) - 1);

        // Shortened const declaration
        // In reality it sets the generics: FieldValue<u32, Register>
        // And as they are constants, sets the types before the value
        pub const Input = FieldValue::new(mask , 15, 0b000);
        pub const Output = FieldValue::new(mask, 15, 0b001);
        pub const AltFunc0 = FieldValue::new(mask, 15, 0b100);

        pub const SET = FieldValue::new(mask, 15, mask);
        pub const CLEAR = FieldValue::new(mask, 15, 0);

        impl TryFromValue<u32> for Value {
            // [ ..snip.. ]
        }

        impl From<Value> for FieldValue<u32, Register> {
            // [ ..snip.. ]
        }
    }
}
```

### Register structs

On the macro `register_structs!`, we used the previously defined fields and modules like so:

``` rust
register_structs! {
    RegisterBlock {
        (0x00 => _unused1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _unused2),
        (0x94 => @END),
    }
}
```

This generates a `struct RegisterBlock`, containing an attribute for each data we defined for it.

The unused blocks are replaced with an array of the expected size (computed as `end - start`),
and our `GPFSEL1` register has the type `ReadWrite` over the generated `GPFSEL1::Register` struct.

``` rust
#[repr(C)]
struct RegisterBlock {
    _unused1: [u8; 0x04 - 0x00],
    GPFSEL1: ReadWrite<u32, GPFSEL1::Register>,
    _unused2: [u8; 0x94 - 0x08],
}
```

> Note that `repr(C)` here allows to respect the order, size and alignment of the fields inside this struct.  
The binary representation of the struct's fields will then match the register addresses and fields that we define.  
See more on [the official documentation of repr(C)][repr_c_docs]

And that's pretty much it !  
All we have to do now is create our register to the expected memory address, and use it !

But how do we get a clean *Rust type* from the raw memory address ?  
Thanks to `#[repr(C)]`, we can directly map the `RegisterBlock` structure into a memory address,
the fields will be provisionned by their binary value stored at this address directly. 

To map the type `RegisterBlock` struct to the address `0x7E200_0000`, we need to:
- Convert the address to a [rust pointer][rust_ptr_doc] (not to confuse with a reference)
- Dereference this pointer, pointing out to the compiler that we expect a `RegisterBlock` type

The pointer type we use here is a `*const _`, it means that the pointer will not move in memory (`const`),
the **_** means that the type will be determined using the [inference system][rust_inference], here the return type.

Dereferencing the pointer `*(ptr)` is an unsafe operation, and as we type the var as `RegisterBlock`,
we get the expected type, containing the expected values stored at the expected address !

``` rust
let start_addr = 0x7E200000;
let regblock : RegisterBlock = unsafe { *(start_addr as *const _) };
regblock.GPFSEL1.write(GPFSEL1::FSEL15::Input);
```

> The memory address `0x7E20_0000` (found on page 90) we passed corresponds to the start of all the registers related to the GPIO.
From this *base address*, an offset of `0x04` will link to the memory address `0x7E20_0004`, which leads to the register `GPFSEL1`

However, there is a case where the user could mess up the inputs in the macro, and **it wouldn't
be detected**, up until we map the type `RegisterBlock` to the start address.

In order to detect this at compile time, the macro generates a `const` which will be evaluated
**at compile time**, and any `panic!` in it will trigger a failed build.

These tests will check the [memory alignment][memory_alignment] of the offsets given, check for
any overlap and make sure the whole address space is covered.

I will not go into details of the tests, but it's basically a nested `const` definition with panics inside.

``` rust
const _: () = {
    const SUM_MAX_ALIGN: (usize, usize) = {
        const SUM_MAX_ALIGN: (usize, usize) = {
            const SUM_MAX_ALIGN: (usize, usize) = {
                const SUM_MAX_ALIGN: (usize, usize) = (0, 0);
                const SUM: usize = SUM_MAX_ALIGN.0;
                const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
                if !(SUM == 0x00) {
                    panic!("message");
                }
                (0x04, MAX_ALIGN)
            };
            const SUM: usize = SUM_MAX_ALIGN.0;
            const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
            if !(SUM == 0x04) {
                panic!("message");
            }
            const ALIGN: usize = core::mem::align_of::<ReadWrite<u32, GPFSEL1::Register>>();
            if !(SUM & (ALIGN - 1) == 0) {
                panic!("message");
            }
            const NEW_SUM: usize = SUM + core::mem::size_of::<ReadWrite<u32, GPFSEL1::Register>>();
            if !(NEW_SUM == 0x08) {
                panic!("message");
            }
            const NEW_MAX_ALIGN: usize = if ALIGN > MAX_ALIGN {
                ALIGN
            } else {
                MAX_ALIGN
            };
            (NEW_SUM, NEW_MAX_ALIGN)
        };
        const SUM: usize = SUM_MAX_ALIGN.0;
        const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
        if !(SUM == 0x08) {
            panic!("message");
        }
        (0x94, MAX_ALIGN)
    };
    const SUM: usize = SUM_MAX_ALIGN.0;
    const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
    if !(SUM == 0x94) {
        panic!("message");
    }
    const STRUCT_SIZE: usize = core::mem::size_of::<RegisterBlock>();
    const ALIGNMENT_CORRECTED_SIZE: usize = if 0x94 % MAX_ALIGN != 0 {
        0x94 + (MAX_ALIGN - (0x94 % MAX_ALIGN))
    } else {
        0x94
    };
    if !(STRUCT_SIZE == ALIGNMENT_CORRECTED_SIZE) {
        panic!("message");
    }
};
```

# Conclusion

In the past, I already attempted to write Rust on embedded systems:
- On an [ESP32 board](https://github.com/litchipi/esp32rs) which was uneasy because of the unstable support
- On the [STM32F103 Bluepill board](https://github.com/litchipi/nix_rust_bluepill_base) with Rust and Nix
- Made a [library to build a GBA game](https://github.com/litchipi/rusty_gbadev) that works on an emulator

All of them were fun to do, and I'm getting more and more used to embedded Rust, it was hacky when I first started playing with it,
it now feels much more mature and I'm happy about this !

**Big big thanks** to [@andre-richter](https://github.com/andre-richter) for his tutorial on [Rust bare metal programming on Raspberry Pi][raspi_rust_tuto]  

**Thank you** for reading this post !  

As usual, feel free to correct me, contact me on Mastodon or email, or browse this blog !  
Take care <3

> Thanks to `Romain KELIFA` for pointing out a typo, and an unclear passage of the article.

[raspi_rust_tuto]: https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/tree/master 
[rpi3_cpu_datasheet]: https://datasheets.raspberrypi.com/bcm2835/bcm2835-peripherals.pdf
[rpi_dt]: https://github.com/raspberrypi/firmware/blob/master/extra/dt-blob.dts
[tock_registers_crate]: https://crates.io/crates/tock-registers
[cargo_expand]: https://crates.io/crates/cargo-expand
[memory_alignment]: https://en.wikipedia.org/wiki/Data_structure_alignment
[repr_c_docs]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprc
[rust_ptr_doc]: https://doc.rust-lang.org/std/primitive.pointer.html
[rust_inference]: https://doc.rust-lang.org/rust-by-example/types/inference.html
