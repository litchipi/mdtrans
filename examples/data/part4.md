In this post, we're going to clone the parent process of our container into a child process.

Before that can happend, we'll set up the ground by preparing some inter-process communication
channels allowing us to interact with the child process we're going to create.

# Inter-process communication (IPC) with sockets
## Introduction to IPC
When it comes to inter-process communication (or IPC for short), the **Unix domain sockets** or
"same host socket communication" is the solution.
They differ from the "network socket communication" kind of sockets that are used to perform
network operations with remote hosts.

You can find a really nice article about sockets and IPC [here][ipctuto]
It consists in practice of a file (*Unix philosophy: everything is a file*)
In which we're going to read or write, to transfer information from one process to another.

For our tool, we don't need any fancy IPC, but we want to be able to transfer simple boolean
to / from our child process.

## Create a socketpair
Creating a pair of sockets will allow us to give one to the child process, and one to the
parent process.

![a pair of sockets](/images/container_in_rust/sockets.png)

This way we'll be able to transfer raw binary data from one process to the other the same way
we write binary data into a file stored in a filesystem.

Let's create a new file `src/cli.rs` containing everything related to IPC:
``` rust
use crate::errors::Errcode;

use std::os::unix::io::RawFd;
use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag, send, MsgFlags, recv};

pub fn generate_socketpair() -> Result<(RawFd, RawFd), Errcode> {
    match socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC)
        {
            Ok(res) => Ok(res),
            Err(_) => Err(Errcode::SocketError(0))
    }
}
```

We create a `generate_socketpair` function in which we call the `socketpair` function, which is
the [standard Unix way of creating socket pairs][man-socketpair], but called from Rust.

- `AddressFamily::Unix`: We are creating a Unix domain socket
(see [all AddressFamily variants for details][AddressFamily-docs])

- `SockType::SeqPacket`: The socket will use a communication semantic with packets and fixed length datagrams.
(see [all SockType variants for details][SockType-docs])

- `None`: The socket will use the default protocol associated with the socket type.

- `SockFlag::SOCK_CLOEXEC`: The socket will be automatically closed after any syscall of the
`exec` family. (see [Linux manual for `exec` syscalls][man-exec])

As we use a new `Errcode::SocketError` variant, let's add it to `src/errors.rs` now:
``` rust
 pub enum Errcode{
    // ...
    SocketError(u8),
 }
```

## Adding the sockets to the container config

When creating the configuration, let's generate our socketpair and add it to the `ContainerOpts`
data so the child process can access to it easily.
In the `src/config.rs` file:
``` rust
use crate::ipc::generate_socketpair;

// ...
use std::os::unix::io::RawFd;
#[derive(Clone)]
pub struct ContainerOpts{
    // ...
    pub fd:         RawFd,
    // ...
}
```

Also let's modify the `ContainerOpts::new` function so it returns the sockets along with the
constructed `ContainerOpts` struct, the parent container needs to get access to it.
``` rust
impl ContainerOpts{
    pub fn new(command: String, uid: u32, mount_dir: PathBuf) -> Result<(ContainerOpts, (RawFd, RawFd)), Errcode> {
        let sockets = generate_socketpair()?;
        // ...
        Ok((
            ContainerOpts {
                // ...
                fd: sockets.1.clone(),
            },
            sockets
        ))
    }
 }
```

## Adding to the container, setting up the cleaning
In our container implementation, let's add a field in the `Container` struct to
be able to access the sockets more easily.
In the file `src/container.rs`:

``` rust
use nix::unistd::close;
use std::os::unix::io::RawFd;
// ...

pub struct Container{
    sockets: (RawFd, RawFd),
    config: ContainerOpts,
 }

impl Container {
    pub fn new(args: Args) -> Result<Container, Errcode> {
        let (config, sockets) = ContainerOpts::new(
            // ...
            )?;
        Ok(Container {
            sockets,
            config,
        })
    }
}
```

As sockets  requires some cleaning before exit, let's close them in the `clean_exit` function.

``` rust
pub fn clean_exit(&mut self) -> Result<(), Errcode>{
    // ...
    if let Err(e) = close(self.sockets.0){
        log::error!("Unable to close write socket: {:?}", e);
        return Err(Errcode::SocketError(3));
    }

    if let Err(e) = close(self.sockets.1){
        log::error!("Unable to close read socket: {:?}", e);
        return Err(Errcode::SocketError(4));
    }
    // ...
}
```

## Creating wrappers for IPC

To ease the use of the sockets, let's create two wrappers to ease the use.
We only want to transfer boolean, so let's create a `send_boolean` and `recv_boolean` function
in `src/ipc.rs`:
``` rust
pub fn send_boolean(fd: RawFd, boolean: bool) -> Result<(), Errcode> {
    let data: [u8; 1] = [boolean.into()];
    if let Err(e) = send(fd, &data, MsgFlags::empty()) {
        log::error!("Cannot send boolean through socket: {:?}", e);
        return Err(Errcode::SocketError(1));
    };
    Ok(())
}

pub fn recv_boolean(fd: RawFd) -> Result<bool, Errcode> {
    let mut data: [u8; 1] = [0];
    if let Err(e) = recv(fd, &mut data, MsgFlags::empty()) {
        log::error!("Cannot receive boolean from socket: {:?}", e);
        return Err(Errcode::SocketError(2));
    }
    Ok(data[0] == 1)
}
```
Here it's just some interacting with the `send` and `recv` functions from the `nix` crate, handling
data types conversion, etc... There's nothing much to say about it, but it's still interesting
how we can interact with functions that has a low-level C backend from Rust.

We won't use the wrappers for now, but they'll come handy later.

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step7”][code-step7].   
The raw patch to apply on the previous step can be found [here][patch-step7]



# Cloning a process
In order to regroup everything related to the cloning and management of the child process, let's
create a new module `child` in a file `src/child.rs`. First of all, define the modules in `src/main.rs`:
``` rust
...
mod config;
mod child;
```
We can also create new types of errors to deal with anything going wrong in our
child process generation or anything during the preparation inside the container,
and add them to `src/errors.rs`:
``` rust
pub enum Errcode {
    ...
    ContainerError(u8),
    ChildProcessError(u8),
}
```

## Creating a child process
For now, we create a dummy child function simply echoing the arguments it will execute.
We create the function in `src/child.rs`:
``` rust
fn child(config: ContainerOpts) -> isize {
    log::info!("Starting container with command {} and args {:?}", config.path.to_str().unwrap(), config.argv);
    0
}
```
The child process simply outputs something to stdout, and returns 0 as a signal that nothing went wrong.
We also pass it some configuration in which we'll be able to bundle everything we want our
child process to acknowledge.

Then we create the function cloning the parent process and calling the child, still in `src/child.rs`:
``` rust
use crate::errors::Errcode;
use crate::config::ContainerOpts;

use nix::unistd::Pid;
use nix::sched::clone;
use nix::sys::signal::Signal;
use nix::sched::CloneFlags;

const STACK_SIZE: usize = 1024 * 1024;

pub fn generate_child_process(config: ContainerOpts) -> Result<Pid, Errcode> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let mut flags = CloneFlags::empty();
    // Flags definition
    match clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32)
    )
    {
         Ok(pid) => Ok(pid),
         Err(_) => Err(Errcode::ChildProcessError(0))
    }
}
```
Let's split this code to understand it properly:
- We first allocate a raw array (aka buffer) of size `STACK_SIZE` that we define of size `1KiB`.
This buffer will hold the [stack][whatis-stack] of the child process, note that this is different
from the original C `clone` function (as detailled in [the nix::sched::clone documentation][docs-clone])

- Secondly we will set the flags we want to activate, a complete list of the flags and their simple description is available
[in the nix::sched::CloneFlags documentation][docs-CloneFlags], or directly [in the linux manual for clone(2)][man-clone].
I'll skip the flags definition for their own seperate parts as they deserve some proper explanation.

- We then call the `clone` syscall, redirecting to our `child` function, with our `config` struct
as an argument, the temporary stack for the process, the flags we set, along with the instruction to
send the parent process a `SIGCHLD` signal when the child exits.

- If everything goes well, we get a process ID, or `PID` in short, a number identifying uniquely our
process for the Linux kernel. We return this pid as we will store it in our container struct.

### A word about namespaces
If you don't know what Linux namespaces are, I recommend reading the [Wikipedia article about it][wikipedia-linux-namespaces]
for a quick and somewhat complete introduction.

In one line, a namespace is an isolation provided by the Linux kernel to allow a process in this
namespace to have a different version of a resource than the global system.

In practice:
- **Network namespace**: Have a different network configuration than the whole system

- **Host namespace**: Have a different hostname than the whole system

- **PID**: Use any PID numbers inside the namespace, including the `init` one (PID = 1)

- And many others ...

Check out the [linux manual for namespaces][man-namespaces] for more details about namespaces.

### Setting the flags
Back to our child cloning preparation, each flag will **create a new namespace for the child process**,
for the given namespace. If a flag is not set, usually the namespace the child will be part of will
be the one from the parent process.

Here is the complete code:
``` rust
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWIPC);
    flags.insert(CloneFlags::CLONE_NEWNET);
    flags.insert(CloneFlags::CLONE_NEWUTS);
```

- `CLONE_NEWNS` will start the cloned child in a new `mount` namespace,
initialized with a copy of the namespace from the parent process.   
*Check [the mount-namespaces manual][man-mountns] for more informations*

- `CLONE_NEWCGROUP` will start the cloned child in a new `cgroup` namespace.   
Cgroups are explained a bit later in the tutorial as we will use them to restrict
the capabilities our child process have.   
*Check [the cgroup-namespaces manual][man-cgroupns] for more informations*

- `CLONE_NEWPID` will start the cloned child in a new `pid` namespace.   
This basically mean that our child process will *think* he will have a PID = X,
but in reality in the Linux kernel he will have another one.   
*Check [the pid-namespaces manual][man-pidns] for more informations*

- `CLONE_NEWIPC` will start the cloned child in a new `ipc` namespace.   
Processes inside this namespace can interact with each other, whereas processes outside
cannot through normal `IPC` methods.   
*Check [the ipc-namespaces manual][man-ipcns] for more informations*

- `CLONE_NEWNET` will start the cloned child in a new `network` namespace.   
It will not share the interfaces and network configurations from other
namespaces.   
*Check [the network-namespaces manual][man-networkns] for more informations*

- `CLONE_NEWUTS` will start the cloned child in a new `uts` namespace.   
I cannot explain why the name UTS (*UTS stands for UNIX Timesharing System*),
but it will allow the contained process to set its own hostname and NIS domain name
in the namespace.   
*Check [the uts-namespaces manual][man-utsns] for more informations*

So while creating our child, we will separate its world from the one of the system,
allowing it to modify whatever it wants (at least for the namespaces used) without harming
the configuration of our system.

## Generate the child from the container
Now that we have our clean `generate_child_process` function, we can call it in the `create`
function of our container, and store the resulting `pid` in the struct fields.

In `src/container.rs`, add:
``` rust
use crate::child::generate_child_process;
use nix::unistd::Pid;
use nix::sys::wait::waitpid;

pub struct Container {
    // ...
    child_pid: Option<Pid>,
}

impl Container {
    pub fn new(args: Args) -> Result<Container, Errcode> {
        // ...
        Ok(Container {
            sockets,
            config,
            child_pid: None,
        })
    }

    pub fn create(&mut self) -> Result<(), Errcode> {
        let pid = generate_child_process(self.config.clone())?;
        self.child_pid = Some(pid);
        log::debug!("Creation finished");
        Ok(())
    }
    // ...
}
```

## Waiting for the child to finish
Now that our container contains everything to generate a new clean child process,
we will update the main function to wait for the child to finish.   
In `src/container.rs`:
``` rust
pub fn start(args: Args) -> Result<(), Errcode> {
    if let Err(e) = container.create(){
        // ...
    }
    log::debug!("Container child PID: {:?}", container.child_pid);
    wait_child(container.child_pid)?;
    // ...
}
```

This way, the container generate the child process using the arguments we give to it, then
hold and wait for the child to end before quitting.

The function `wait_child` is defined in `src/container.rs` like so:
``` rust
pub fn wait_child(pid: Option<Pid>) -> Result<(), Errcode>{
    if let Some(child_pid) = pid {
        log::debug!("Waiting for child (pid {}) to finish", child_pid);
        if let Err(e) = waitpid(child_pid, None){
            log::error!("Error while waiting for pid to finish: {:?}", e);
            return Err(Errcode::ContainerError(1));
        }
    }
    Ok(())
}
```
This function uses the syscall `waitpid`, from the [manual][man-waitpid]:
> The **waitpid()** system call suspends execution of the calling process until a child specified by
> pid argument has changed state. By default, waitpid() waits only for terminated children,
> but this behavior is modifiable via the options argument, as described below.

As we wait for the termination, we will just pass `None` as options, and return a `Errcode::ContainerError`
error if the syscall didn't finished succesfully.

## Testing
Maybe since the beginning you were wondering why we need `sudo` to run our tests, in the first 7
steps that wasn't necessary, but here as we create new namespaces for our child process, the
`CAP_SYS_ADMIN` capacity is needed (See the [manual for capabilities][man-capabilities] or
[this article from LWN][lwn-capabilities]).

Here's the output we can get from testing this step:
```
[2021-11-12T08:52:17Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2021-11-12T08:52:17Z DEBUG crabcan::container] Linux release: 5.11.0-38-generic
[2021-11-12T08:52:17Z DEBUG crabcan::container] Container sockets: (3, 4)
[2021-11-12T08:52:17Z DEBUG crabcan::container] Creation finished
[2021-11-12T08:52:17Z DEBUG crabcan::container] Container child PID: Some(Pid(134400))
[2021-11-12T08:52:17Z DEBUG crabcan::container] Waiting for child (pid 134400) to finish
[2021-11-12T08:52:17Z INFO  crabcan::child] Starting container with command /bin/bash and args ["/bin/bash"]
[2021-11-12T08:52:17Z DEBUG crabcan::container] Finished, cleaning & exit
[2021-11-12T08:52:17Z DEBUG crabcan::container] Cleaning container
[2021-11-12T08:52:17Z DEBUG crabcan::errors] Exit without any error, returning 0
```

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step8”][code-step8].   
The raw patch to apply on the previous step can be found [here][patch-step8]

[code-step7]: https://github.com/litchipi/crabcan/tree/step7/
[patch-step7]: https://github.com/litchipi/crabcan/compare/step6..step7.diff
[code-step8]: https://github.com/litchipi/crabcan/tree/step8/
[patch-step8]: https://github.com/litchipi/crabcan/compare/step7..step8.diff

[ipctuto]: https://opensource.com/article/19/4/interprocess-communication-linux-networking
[man-socketpair]: https://man7.org/linux/man-pages/man2/socketpair.2.html
[man-exec]: https://man7.org/linux/man-pages/man3/exec.3.html
[man-clone]: https://man7.org/linux/man-pages/man2/clone.2.html
[man-namespaces]: https://man7.org/linux/man-pages/man7/namespaces.7.html

[man-mountns]: https://man7.org/linux/man-pages/man7/mount_namespaces.7.html
[man-cgroupns]: https://man7.org/linux/man-pages/man7/cgroup_namespaces.7.html
[man-pidns]: https://man7.org/linux/man-pages/man7/pid_namespaces.7.html
[man-ipcns]: https://man7.org/linux/man-pages/man7/ipc_namespaces.7.html
[man-networkns]: https://man7.org/linux/man-pages/man7/network_namespaces.7.html
[man-utsns]: https://man7.org/linux/man-pages/man7/uts_namespaces.7.html
[man-waitpid]: https://linux.die.net/man/2/waitpid
[man-capabilities]: https://man7.org/linux/man-pages/man7/capabilities.7.html

[docs-clone]: https://docs.rs/nix/0.23.0/nix/sched/fn.clone.html
[docs-CloneFlags]: https://docs.rs/nix/0.23.0/nix/sched/struct.CloneFlags.html
[whatis-stack]: http://www.c-jump.com/CIS77/ASM/Stack/lecture.html

[AddressFamily-docs]: https://docs.rs/nix/0.22.1/nix/sys/socket/enum.AddressFamily.html
[SockType-docs]: https://docs.rs/nix/0.22.1/nix/sys/socket/enum.SockType.html

[wikipedia-linux-namespaces]: https://en.wikipedia.org/wiki/Namespace
[lwn-capabilities]: https://lwn.net/Articles/486306/
