On January 17th 2023, **X41** and **Gitlab** published a report of the source
code audit they performed on Git (funded by the **OSTIF** foundation).

This post is based on the (*great*) report available [here][report_url] and aims
to investigate how Rust mitigates some of the vulnerabilities shown in this report,
but also to put some light on **what it doesn't** mitigate by itself, and how a programmer
can address these issues using good practices.

> The role of these kinds of studies are primordial, and the **OSTIF** allows to
> fund such initiatives.  
> Please for the sake of great opensource software and computer security
> **consider [making a donation][ostif_donation]**.

If I say anything that is wrong / oversimplified, *do tell me please* and I will correct
this article in consequence.

# Detailing some vulnerabilities

The explanations are *intentionally kept simple*, please refer to the corresponding
report section for more informations.  
I'll be focusing on the basics (that should prevent me from saying too many
wrong statements), and how these issues could be seen in a program made in Rust.

## GIT-CR-22-01

This vulnerability (described at section *4.1.1*) is an *Uncontrolled Resource
Consumption* ([CWE 400][cwe_400]), leading to a possible Denial of Service.  
The issue is caused by this loop occurring in one of the functions:

``` c
while (slen > 0) {
    int len = slen;      // Here if slen is too big, it loops backs to 0
    // Allocate some memory
    slen -= len;        // len = 0, slen > 0, so the loop goes infinite
}
```

When the input text grows, the value of `slen` does as well, and researchers
suceeded to allocate **2.5GB** of memory using a *30MB* file.

Let's try to reproduce the same kind of loop in Rust now:

``` rust
let mut slen: u64 = (u32::MAX as u64) + 1;
while slen > 0 {
    let len: u32 = slen as u32;
    println!("Some memory allocation");
    slen -= len as u64;
}
```

This code compiles without any warning and gives you a nice infinite loop
indeed.  
However note the *2 casts* that we are required to perform in order to get to this
point, it's much *easier to notice* that something may break here, but on a large
codebase with an example much more complex, *it may still be occurring*.

Lesson number one: **Rust doesn't protect from casting overflows** if you cast
naively using `as`. However you can use the `try_into` function for casting
that will return a `Result<T, T::Error>` triggered if such things happen.

## GIT-CR-22-02

This vulnerability (described at section *4.1.2*) is an *Out of Bound Read*
([CWE 125][cwe_125]), allowing possible sensitive data reading, or even
a buffer overflow.

When dealing with strings, the computer maps the data in memory as:

```
Letters: "r"  "u"  "s"  "t"
Bytes:   0x72 0x75 0x73 0x74 0x00
```

Notice the `0x00` byte at the end ? That's how the computer can detect the end of
the string.  
As the numeric value for this byte is `0`, in C we detect it using:

``` c
int next_char = get_next_char(some_string);
if (!next_char) {
    // The char is the end of the string
}
```

However imagine `get_next_char` doesn't return the value, but *a pointer* to this
value, then `!char` doesn't check if the value is `0` anymore, but it checks if the
*pointer is a `NULL` pointer*, which **it won't be** even if its value may be `0`.

This vulnerability exists because the code *forgets to dereference*
the returned pointer (pointing to somewhere in the string) when performing a
condition.
Because this condition will be *always true*, it allows the execution of a
function with invalid inputs, leading to the vulnerable behavior.

In Rust, you have to use an explicit `unsafe` block to perform operations on raw
pointers like this, and that doesn't give you the "full power".  
As I am too unfamiliar with *unsafe Rust*, I won't try to reproduce this here.

You should **not use unsafe** Rust as long as you don't have a *very specific reason*
to use it.

## GIT-CR-22-03

This vulnerability (described at section *4.1.3*) is an *Integer overflow
to Buffer overflow* ([CWE 680][cwe_680]), that could lead to code execution.

The issue here is that in Windows 64bit, the size of an `unsigned long` is
**4 bytes**, whereas it is **8 bytes** on Linux 64bit.  
As `git` has been created with *Linux in mind*, in the following code, something
goes wrong:

``` c
size_t msg_A_len = get_message_length(msg_a);
size_t msg_B_len = get_message_length(msg_b);
unsigned long new_buffer_len = msg_A_len + msg_B_len + 2;
```

Indeed, we can overflow the bounds of the `unsigned long new_buffer_len`,
looping back to `0`. Now we can get big inputs that make the `new_buffer_len`
variable small.  
Now imagine the remaining of the code would be something like:

``` c
char* new_buffer = malloc(new_buffer_len);
memcpy(new_buffer, msg_a, msg_A_len);
memcpy(new_buffer + msg_A_len, msg_b, msg_B_len);
```

In the case when `new_buffer_len < (msg_A_len + msg_B_len)`, that would mean
we just *wrote more bytes* into memory than the allocated *memory was able
to contain*, we just **overflowed** the buffer.

Now let's do something similar in Rust shall we ?

``` rust
println!("size of usize: {}", std::mem::size_of::<usize>());
// size of usize: 8

let msg_a_len: u64 = u64::MAX >> 1;
let msg_b_len: u64 = u64::MAX >> 1;
assert_eq!(msg_a_len + msg_b_len + 1, u64::MAX);

let new_buffer_len: usize = (msg_a_len as usize) + (msg_b_len as usize) + 3;
// debug compilation:     thread panick here because of integer overflow

let some_array: Vec<u8> = Vec::with_capacity(new_buffer_len);
println!("{}", some_array.capacity());
// release compilation:         1
```

Several things on this:

- I am running on a `x86_64` machine, meaning `usize` is the same length as an `u64`,
so this code does not reproduce what actually happens in this vulnerability.
- Notice the casts that we are required to do in order to make this compile, *this
is a hint* that the programmer have to check his bounds.
- The overflow protection is only on in `debug` mode (and can be set/unset in the
[compilation profile][cargobook_compilation_profile])
- This code **cannot be exploited** in safe code to perform a *memory overflow*,
because you would have to use a `Vec` here, which has *all the safeguards* to
not overflow. Arrays are *not possible* here as their size has to be known
at compile time.
- The size of each numeric type (except `usize`) *is obvious* to the developer and
*doesn't change* across platforms, this is by itself a great protection as long
as we pay attention to the types we use. (`let size: i32` is not a good practice
at all.)

This kinds of issues can **definitly happen** in Rust if you use `as` castings all
the time, and even if the memory size of variable is much more simple in
Rust than in C, `usize` size in memory is arch-dependant
(as described in the [usize type documentation][usize_type_doc]).

The issue may exist in Rust, but the **memory vulnerability** doesn't (at least
in safe Rust), attempts to write something and run some code would lead to the
program termination.

## GIT-CR-22-04

This vulnerability (described at section *4.1.4*) is a *Synchronous Access
of Remote Resource without Timeout* ([CWE 1088][cwe_1088]), leading to
a possible Denial of Service.

This issue is really simple to understand. When initiating a new connection
for a `git clone` operation, no timeout is set, meaning that if the remote
endpoint doesn't answer, the connection is kept open.

If an attacker open X connections to endpoints he controls (and that doesn't
send any data), then the connections are kept active in `git`, the resources
aren't freed, leading to a Denial of Service.

Rust is not really protected by that kinds of things, and it's to the
attention of the developer to pay attention to *always* put some kind
of *boundaries* to the connections, whether it is a hard limit of
simultaneously opened connections, a timeout, etc ...

Timeouts can be annoying to set up, but they may really be worth it as there
is nothing as unexpected as a bad Internet connection, a crash from a distant
server, or any kind of I/O scenario a sane programmer wouldn't think of.

Even a long one is useful for these cases.

## GIT-CR-22-05

This vulnerability (described at section *4.1.5*) is an *Inefficient Regular
Expression Complexity* ([CWE 1333][cwe_1333]), leading to a possible
Denial of Service by consumming excessive CPU resources.

This vulnerability happens because somewhere in the code, the user input
gets interpreted as a regular expression.  
Using this, an attacker can pass a nasty Regexp that cause a denial of
Service, called *ReDos* in that case.

See the [OWASP article][owasp_redos] about this kind of attacks.

For the [regex][regex_crate_docs] crate, the reference when dealing with regular
expressions in Rust, the security is taken seriously and some features at the
root cause of this kind of problems are simply not implemented. That mean
that the crate is less powerful than other systems out there, but it is
a tradeoff that the developers chose to make.

See the "[Untrusted inputs][regex_docs_untrused_inp]" section from the docs
of the crate for more details

## GIT-CR-22-06

This vulnerability (described at section *4.1.6*) is a *Heap-based Buffer
Overflow* ([CWE 122][cwe_122]), leading in the worst case scenario to
arbitrary code execution.

This is also a vulnerability caused by the overflow of the type `int`
when getting numbers bigger than its bounds and loops back to the
minimum bound, in this case negative:

``` c
size_t len = get_length(buffer);
size_t padding = get_padding(input_string);

int offset = padding - len;
memcpy(buff + offset, input_string, len);
```

However here this is much more problematic as it allows to set
a negative offset, and so perform a `memcpy` operation **before**
the start of the buffer, and write data controlled by the attacker.

Heap overflows may not be as bad as Stack overflows, but they do have
really nasty exploit possible (see [CTF101 article][ctf101_heap_exploit] about it).

Now would that kind of things be possible in a world covered in (*safe*) Rust ?  
We saw that Rust doesn't always protect against numeric types overflow.

However you would have to choose between different types to use, meaning that
for this situation to happen, you would have to explicitly write `i64 offset`,
as `usize` is unsigned and neither are `u8 u16 u32 u64`.

I decided to count this as a *protection*, as the amount of work required
to make this behavior happen assures that you brought it on yourself.  
Using `unsafe`, it is possible to get similar problems leading to possible
vulnerabilities as well, for example in this code:

``` rust
let input_string = String::from("this is longer than the length of the buffer");
let strlen: usize = input_string.len();

let bufflen: usize = 10;
let buffer = String::with_capacity(bufflen);

let offset: i64 = (bufflen as i64) - (strlen as i64);
let ptr = buffer.as_mut_ptr();
unsafe {
    std::ptr::copy(input_string.as_ptr(), ptr.offset(offset), input_string.len());
}
```

> Thanks to `u/pluuth` for pointing out the correct unsafe code here,
> I previously thought it was much more complex to get an issue like this to appear
> in Rust. I still count it as "protected" because of the mandatory `unsafe` block
> and the type casts you need to use.

## GIT-CR-22-07

This vulnerability (described at section *4.1.7*) is another *Heap-based
Buffer Overflow* ([CWE 122][cwe_122]), similarly leading in the worst
case scenario to arbitrary code execution.

This is yet another `int` type overflow when handling big inputs
(displaying how important this is), however this vulnerability is really
**critical** as now an attacker can commit a malicious `.gitattributes` file
into a remote repository, and the vulnerability will be triggered to
anybody trying to `clone` or `pull` the repository.

Here the recommendation for this issue is to use a `size_t` type in order
to prevent the integer overflow, however it's also pointed out to **limit
the size** of the lines in the `.gitattributes` file.

However as this is can be only exploited for memory manipulation, this
is where (safe) Rust protects us. It becomes critical because of the
possible arbitrary code execution following, however this could be
used in Rust to create *infinite loops*, trigger conditions, etc ...

## GIT-CR-22-08

This vulnerability (described at section *4.1.8*) is an *Uncontrolled
Resource Consumption* ([CWE 400][cwe_400]), leading to a possible
Denial of Service.

The report isn't really clear about this, but when applying a patch, this
code gets triggered:

``` c
// apply.c:4687
static int apply_patch(struct apply_state *state,
		       int fd,
		       const char *filename,
		       int options)
{
    // ...
    offset = 0;
    while (offset < buf.len) {
            nr = parse_chunk(state, buf.buf + offset, buf.len - offset, patch);
            if (nr < 0) {
                // Error case
            }
            // Some operations
            offset += nr;
    }
    // ...
}
```

The vulnerability here comes from an issue making the `parse_chunk` returns 0, resulting
in an infinite loop.

It's caused by yet another integer overflow, when parsing a *binary patch* file.
With a header / payload long enough, you can overflow the variables, and the return
value from the function gets overflowed:

``` c
// apply.c:2124
static int parse_chunk(struct apply_state *state,
                       char *buffer, unsigned long size, struct patch *patch)
{
    // ...
    return offset + hdrsize + patchsize;
}
```

As an `int` can be overflowed to a negative value, a malicious patch can return
`0` here, and performing a `git apply` over the patch would result in an infinite
loop, and a Denial of Service.

This is a case that *could totally apply* to a Rust code if we are not careful enough,
the best protection is to **use proper numeric types** to ensure a size doesn't get
negative, but you should also *learn to notice* the conditions that would make any
loop go infinite, and check for these conditions.

# Good security practices in Rust code

Remember that the Rust "safety" is only relative and under particular conditions,
it's not the same if you are writing for embedded systems, or in the Linux kernel
(as [Linus explained][linus_lkml] in some of the mails), and it only works if *you
set up* everything Rust needs to achieve safety

> And the *reality* is that there are no absolute guarantees. Ever.  
> The "Rust is safe" is not some kind of absolute guarantee of code safety.
> Never has been.

Rust is implemented so it eliminates undefined behaviors, and handles the "wrong
answer" case by returning an error, or panicking. This is a choice that makes
total sense when building a software / an application, but it's not a universal
"best way to do", it depends on what you do.

> Not completing the operation at all, is *not* really any better than
> getting the wrong answer, it's only more debuggable.  
> [ ... ]  
> So this is something that I really *need* the Rust people to
> understand. That whole reality of "safe" not being some absolute
> thing, and the reality that the kernel side *requires* slightly
> different rules than user space traditionally does.

Let's review some good practices (generally speaking) in Rust to ensure we don't
hit too much errors, or create some vulnerabilities.

## Casting overflow

For a quick and dirty cast, `as` is fine, as long as **you are sure** of
your bounds.

However a *best practice* is to never use it and rely on the `From` and `TryFrom`
traits.  
If you *upcast* `u32` to `u64`, you can use `.into()` as `From<u32>` is
implemented for `u64`, and when you *downcast* `u64` to `u32`,
use `.try_into()` instead, it will return an error if you overflow
the bounds of the integer.  
The performance cost for the usage of these is *negligable* / null,
so you should always use them for a clean and secure code.

As `usize` size in memory is arch-dependant (see [the docs][usize_type_doc]),
I advice to use numeric variable types that have a fixed memory space,
like `u64`, `i32` or `f32`, as much as possible to reduce the possibility
of an integer overflow panick.  
That is even more true if you expect your code to run on different architectures.

## Limit input size

This isn't specific to Rust, but why using `u64` and checking for overflows
everywhere when you can simply *limit* any input length is `< u8::MAX` ?
(Or whatever type you use)

Input sanitization is important, and the size of the input is one aspects of it,
you should never overlook it as it can lead to nasty behaviors.
You may think that "it will never happen, to have a blog post title larger than
65536 characters", but an attacker *will* think of this case, and **break** your
code.

## Unsafe Rust

If you use some *unsafe* in your Rust code, be **very careful** of what is written
inside, make it *as little* as possible, as *tested* as possible, and only if you
cannot make it using *another way*.

It may be ok to use unsafe blocks in these situations:

- Writing to a memory address in embedded systems and kernel code
- Importing code from another programming langage, like C
- Having a global mut pointer, in single threaded application, and only if really
necessary
- When implementing the [Quake 3 inverse square root][quake3_inv_sqrt] function,
or anything similarly esoteric

If something breaks in an unexpected way, the unsafe parts of the code must become
primary suspects, so keep it as clear, simple and documented as possible.

If the use of `unsafe` really improves the performances, add some benchmarks in order
to prove it, and if one day the gap between the `safe` and `unsafe` implementation
is getting close, consider moving back to the all-safe implementation.

In any case, if you **do** have `unsafe` in your code, *test it extensively*, plug
your CI to perform the tests before any merge, and make sure all of it is well tested.

## Limit the scale of your software

- If you *add a data* struct to some `Vec` everytime someone connects to your server,
this is a resource consumption.
- If you *start a thread* performing some kind of *computation* when someone connects
to your server, this is a ressource consumption.

In both cases (and many other kind of examples), you need to think of
"What happens if the whole Earth wants to connect to my app ?"

Answer is, your server will crash, or melt. So put in place **some limits** to the
number of users, or an (inexpensive) waiting queue from which users will be
redirected once a "resource unit" will be available.  
A lot of resources are available online to learn how to deal with these issues,
so I won't give you any naïve advices here, do your researches.

I personally think that you can have applications serving millions of users running
on a tiny server, if you smartly *designed the way your resources are being used*,
and put in place ways to *improve* the behavior of your software **under stress**.

# Closing thoughts

## Rust's protection

Rust by itself, when not using any *unsafe*, is preventing against some vulnerabilities,
including **2 Critical**, *1 high* and 1 medium.

However it didn't protect against 4 Low vulnerabilities.

Note that most of the protection was not because the vulnerability didn't occur,
but more because *it's not exploitable*, or at least with less critical impact.
This is what the rules of Rust concerning memory manipulation protects you from.

However as the issues causing these vulnerabilities can still happen, you can still
have vulnerabilities, and *may have critical ones* if you only rely on "Rust is safe".

Rust *in most cases* is memory safe, but not all exploits are about memory exploitation,
**nothing** is *always* safe, always **doubt** the security of a software, whether
you code it or buy it.  
In code or in life, apply [Defense in depth][wikipedia_defindepth], know the strengths
and weaknesses of the technology you use, and keep your mind open :-)

## Writing secure Rust code

The bottom line is that **you should be careful** when writing code, especially
when handling system's inputs (whether it is human or network / disk).  
This is where *extensive tests* are useful (parametric tests for example),
and in general the more the attacker can control an input in the vulnerable
code, the more he *can exploit it badly*.

You may think that it's only meant for big opensource project and that your
code is OK without all of this, but I think **it's important to train** as these
"good practices" only become automatic *if repeated* enough. So try to think a
little about security when building the next (blazing fast) GNU tool rewritten
in Rust, or anything else really.  
*Get used* to write secure code *by default*.

## Donate to OSTIF

You can make a donation to the OSTIF fund by following [this link][ostif_donation].

The report is accessible [here][report_url] and the summary can be seen on
[X41's website][x41_report_summary].

Once again, if you find anything that is wrong / oversimplified in this article,
**please tell me** so I can correct it right away.

> Special thanks to  
> `u/Rodrigodd_` for pointing out some things to improve in the article  
> `u/Shnatsel` for pointing an imprecision in `GIT-CR-22-03`'s conclusion  
> `u/milliams` for correcting some typos  
> `u/ssokolow` for correcting some typos and grammar mistakes, and details about
> unsafe benchmarking and testing  
> `@teor2345` for giving a precision on casting, allowing to improve the
good practices recommendations on casting.  
> `@myers`, `@Arriv9l` for a correcting a typo  
> `@pepsiman` for correcting several typos over the whole article  

[cwe_400]: https://cwe.mitre.org/data/definitions/400.html
[cwe_125]: https://cwe.mitre.org/data/definitions/125.html
[cwe_680]: https://cwe.mitre.org/data/definitions/680.html
[cwe_1333]: https://cwe.mitre.org/data/definitions/1333.html
[cwe_1088]: https://cwe.mitre.org/data/definitions/1088.html
[cwe_122]: https://cwe.mitre.org/data/definitions/122.html

[report_url]: https://www.x41-dsec.de/static/reports/X41-OSTIF-Gitlab-Git-Security-Audit-20230117-public.pdf
[ostif_donation]: https://ostif.org/donate-to-ostif/
[x41_report_summary]: https://x41-dsec.de/security/research/news/2023/01/17/git-security-audit-ostif/
[quake3_inv_sqrt]: https://en.wikipedia.org/wiki/Fast_inverse_square_root#Overview_of_the_code
[usize_type_doc]: https://doc.rust-lang.org/std/primitive.usize.html
[cargobook_compilation_profile]: https://doc.rust-lang.org/cargo/reference/profiles.html
[regex_crate_docs]: https://docs.rs/regex/latest/regex/
[regex_docs_untrused_inp]: https://docs.rs/regex/latest/regex/#untrusted-input
[owasp_redos]: https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS
[ctf101_heap_exploit]: https://ctf101.org/binary-exploitation/heap-exploitation/
[linus_lkml]: https://lkml.org/lkml/2022/9/19/1105
[wikipedia_defindepth]: https://en.wikipedia.org/wiki/Defense_in_depth_(computing)
