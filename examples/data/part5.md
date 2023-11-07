# Setting the container hostname
A hostname is what identifies our machine compared to every other living on the same network.

It is used by many different networking softwares, for example `avahi` is a software that streams
our hostname in the local network, allowing a command `ssh crab@192.168.0.42` to become
`ssh crab@crabcan.local`, the website `http://localhost:80` to `http://crabcan.local`, etc ...
> Check [the official website of avahi](http://avahi.org/) for more informations

In order to differentiate the operations performed by our software contained from the one
performed by the host system, we will modify its hostname.

## Generate a hostname
First of all, create a new file called `src/hostname.rs`, in which we will write any code
related to the hostname.   
Inside, set two arrays of pre-defined names and adjectives that we'll use together to
generate a stupid random hostname.

``` rust
const HOSTNAME_NAMES: [&'static str; 8] = [
    "cat", "world", "coffee", "girl",
    "man", "book", "pinguin", "moon"];

const HOSTNAME_ADJ: [&'static str; 16] = [
    "blue", "red", "green", "yellow",
    "big", "small", "tall", "thin",
    "round", "square", "triangular", "weird",
    "noisy", "silent", "soft", "irregular"];
```

We then generate a string using some randomness:

``` rust
use rand::Rng;
use rand::seq::SliceRandom;

pub fn generate_hostname() -> Result<String, Errcode> {
    let mut rng = rand::thread_rng();
    let num = rng.gen::<u8>();
    let name = HOSTNAME_NAMES.choose(&mut rng).ok_or(Errcode::RngError)?;
    let adj = HOSTNAME_ADJ.choose(&mut rng).ok_or(Errcode::RngError)?;
    Ok(format!("{}-{}-{}", adj, name, num))
}
```

> We obtain a hostname in the form *square-moon-64*, *big-pinguin-2*, etc ...

As we used a new `Errcode::RngError` to handle errors linked to the randomness functions, we add
this variant to our `Errcode` enum in `src/errors.rs`, along with another one that we'll use later,
the `Errcode::HostnameError(u8)` variant:
``` rust
pub enum Errcode {
    // ...
    HostnameError(u8),
    RngError
}
```

Also, we use the `rand` crate to have randomness in our hostname generation, so we have to add
it to the dependencies of `Cargo.toml`:
``` toml
[dependencies]
# ...
rand = "0.8.4"
```

## Adding to the configuration of the container
Now that we have a way to generate a `String` containing our "random" hostname, we can use it to set our container
configuration in `src/config.rs`:

``` rust
use crate::hostname::generate_hostname;

pub struct ContainerOpts{
    // ...
    pub hostname:   String,
}

// ...

impl ContainerOpts{
    pub fn new(...) -> ... {
        // ...

        Ok((
            ContainerOpts {
                // ...
                hostname: generate_hostname()?,
            },
            // ...
        ))
    }
}
```

And finally, we can create in `src/hostname.rs` the function that will modify the actual hostname
of our *host namespace* with the new one, using the `sethostname` syscall:

``` rust
use crate::errors::Errcode;

use nix::unistd::sethostname;

pub fn set_container_hostname(hostname: &String) -> Result<(), Errcode> {
    match sethostname(hostname){
        Ok(_) => {
            log::debug!("Container hostname is now {}", hostname);
            Ok(())
        },
        Err(_) => {
            log::error!("Cannot set hostname {} for container", hostname);
            Err(Errcode::HostnameError(0))
        }
    }
}
```

> Check the [linux manual][man-sethostname] for more informations on the `sethostname` syscall

## Applying the configuration to the child process

For all the configuration we will apply to the child process, let's create a wrapping functions
setting everything, and add the `set_container_hostname` function call inside it, in `src/child.rs`:

``` rust
use crate::hostname::set_container_hostname;

fn setup_container_configurations(config: &ContainerOpts) -> Result<(), Errcode> {
    set_container_hostname(&config.hostname)?;
    Ok(())
}
```

And we then simply call the configuration function at the beginning of our child process:

``` rust
fn child(config: ContainerOpts) -> isize {
    match setup_container_configurations(&config) {
        Ok(_) => log::info!("Container set up successfully"),
        Err(e) => {
            log::error!("Error while configuring container: {:?}", e);
            return -1;
        }
    }
	// ...
}
```

Note that we cannot "recover" from any error hapenning in our child process, so we simply end it
with a `retcode = -1` along with a nice error message in case a problem occurs.

The final thing to do here is adding to `src/main.rs` the `hostname` module we just created:
``` rust
// ...
mod hostname;
```

## Testing
When testing, we can see the hostname we generated appear:
```
[2021-11-15T09:07:38Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2021-11-15T09:07:38Z DEBUG crabcan::container] Linux release: 5.13.0-21-generic
[2021-11-15T09:07:38Z DEBUG crabcan::container] Container sockets: (3, 4)
[2021-11-15T09:07:38Z DEBUG crabcan::container] Creation finished
[2021-11-15T09:07:38Z DEBUG crabcan::container] Container child PID: Some(Pid(26003))
[2021-11-15T09:07:38Z DEBUG crabcan::container] Waiting for child (pid 26003) to finish
[2021-11-15T09:07:38Z DEBUG crabcan::hostname] Container hostname is now weird-moon-191
[2021-11-15T09:07:38Z INFO  crabcan::child] Container set up successfully
[2021-11-15T09:07:38Z INFO  crabcan::child] Starting container with command /bin/bash and args ["/bin/bash"]
[2021-11-15T09:07:38Z DEBUG crabcan::container] Finished, cleaning & exit
[2021-11-15T09:07:38Z DEBUG crabcan::container] Cleaning container
[2021-11-15T09:07:38Z DEBUG crabcan::errors] Exit without any error, returning 0
```
And running it several times outputs different funny names :D
```
[2021-11-15T09:08:33Z DEBUG crabcan::hostname] Container hostname is now round-cat-221

[2021-11-15T09:08:48Z DEBUG crabcan::hostname] Container hostname is now silent-man-45

[2021-11-15T09:09:01Z DEBUG crabcan::hostname] Container hostname is now soft-cat-149
```

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step9”][code-step9].   
The raw patch to apply on the previous step can be found [here][patch-step9]

# Modifying the container mount point

The mount point is a directory in our system that will be the root, the `/` of our container.
A user can pass to the arguments a directory that will be used as the root of the container.

The process will be done as followed:
- Mount the system root `/` inside the container
- Create a new temporary directory `/tmp/crabcan.<random_string>`
- Mount the user-given directory to the temporary directory
- Perform a *root pivot* over the two mounted directories
- Unmount and delete un-necessary directories

**Keep in mind** that everything we mount / unmount inside the container are isolated from the rest
of the system by the *mount namespace*.

In practise, this isolation keeps separated versions of `/proc/<pid>/mountinfo`, `/proc/<pid>/mountstats`
and `/proc/<pid>/mounts/`, that describes what is mounted where, how, etc ...

>See [proc(5) linux manual](https://man7.org/linux/man-pages/man5/proc.5.html) or [mount_namespace linux manual](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) for more precisions on this.

## Preparing the implementation

As we will create a `src/mounts.rs` file containing a function `setmountpoint`, let's create
everything right now so we can focus on our directories later on.

In `src/child.rs`, let's write our `setmountpoint` function as part of the container
configuration process:
``` rust
use crate::mounts::setmountpoint;
fn setup_container_configurations(config: &ContainerOpts) -> Result<(), Errcode> {
    // ...
    setmountpoint(&config.mount_dir)?;
    Ok(())
}
```

Then in `src/container.rs`, we add a new element in the `clean_exit` function:

``` rust
use crate::mounts::clean_mounts;

impl Container {
    // ...
    pub fn clean_exit(&mut self) -> Result<(), Errcode>{
        // ...
        clean_mounts(&self.config.mount_dir)?;
    }
}
```

We then add a new error variant in our `Errcode` enum in `src/errors.rs`:

``` rust
pub enum Errcode {
    // ...
    MountsError(u8),
}
```

Finally, we use the `src/mounts.rs` file as a module in our project.
In `src/main.rs`:

``` rust
// ...
mod mounts;
```

## Remounting the root `/` privately

Now to the real meat ! Let's create the `src/mounts.rs` file and add the following:

``` rust
use crate::errors::Errcode;
use std::path::PathBuf;

pub fn setmountpoint(mount_dir: &PathBuf) -> Result<(), Errcode> {
    log::debug!("Setting mount points ...");
    Ok(())
}

pub fn clean_mounts(_rootpath: &PathBuf) -> Result<(), Errcode>{
    Ok(())
}
```

We want to remount the root `/` of our filesystem with the `MS_PRIVATE` flag which will
prevent any mount operation to be propagated.

> See [this LWN article](https://lwn.net/Articles/689856/) for more explanations on what the
`MS_PRIVATE` flag is about,
and [this other LWN article](https://lwn.net/Articles/690679/) for an example.

To do this, we will create the `mount_directory` function that is essentially a wrapper
around the [`mount` syscall](https://man7.org/linux/man-pages/man2/mount.2.html) provided by the [`nix` crate](https://docs.rs/nix/latest/nix/).

``` rust
use nix::mount::{mount, MsFlags};

pub fn mount_directory(path: Option<&PathBuf>, mount_point: &PathBuf, flags: Vec<MsFlags>) -> Result<(), Errcode>{
    // Setting up the mount flags
    let mut ms_flags = MsFlags::empty();
    for f in flags.iter(){
        ms_flags.insert(*f);
    }
    // Calling the syscall, handling errors
    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(p) = path{
                log::error!("Cannot mount {} to {}: {}",
                    p.to_str().unwrap(), mount_point.to_str().unwrap(), e);
            }else{
                log::error!("Cannot remount {}: {}",
                    mount_point.to_str().unwrap(), e);
            }
            Err(Errcode::MountsError(3))
        }
    }
}
```

And call it inside our `setmountpoint` function:

``` rust
pub fn setmountpoint(mount_dir: &PathBuf) -> Result<(), Errcode> {
    // ...
    mount_directory(None, &PathBuf::from("/"), vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE])?;
    // ...
}
```

## Mount the new root

Now let's mount the directory provided by the user so we can *pivot root* later.
I wont go into deep details of every line of code as this is simply calling library functions.

First, let's create a `random_string` function that returns, well, a random string.

``` rust
// Taken from https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
use rand::Rng;

pub fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    let name: String = (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    name
}
```
That will allow us to generate a random directory name easily.

Then, after we have our random directory name, let's create that directory:

``` rust
use std::fs::create_dir_all;

pub fn create_directory(path: &PathBuf) -> Result<(), Errcode>{
    match create_dir_all(path) {
        Err(e) => {
            log::error!("Cannot create directory {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountsError(2))
        },
        Ok(_) => Ok(())
    }
}
```

And finally let's tie everything together in our `setmountpoint` function:

``` rust
pub fn setmountpoint(mount_dir: &PathBuf) -> Result<(), Errcode> {
    // ...
    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!("Mounting temp directory {}", new_root.as_path().to_str().unwrap());
    create_directory(&new_root)?;
    mount_directory(Some(&mount_dir), &new_root, vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE])?;
    // ...
}
```

We mounted our user-provided `mount_dir` to the mountpoint `/tmp/crabcan.<random_letters>`, this will
allow us to *pivot root* later and use the `mount_dir` as if it was the real `/` root of the system.

## Pivot the root

Now to the real magic trick ! We set `/tmp/crabcan.<random_letters>` as our new `/` root filesystem,
and we will move the *old* `/` root into a new dir `/tmp/crabcan.<random_letters>/oldroot.<random_letters>`:

*Example:*
```
Outside the container                                          Inside the container
~/container_dir  == mount ==> /tmp/crabcan.12345  == pivot ==> /
                              /                   == pivot ==> /oldroot.54321
```

> See the [linux manual](https://man7.org/linux/man-pages/man2/pivot_root.2.html) for a detailed
explanation of this process

This is how we can do it in the code:

``` rust
use nix::unistd::pivot_root;

pub fn setmountpoint(mount_dir: &PathBuf) -> Result<(), Errcode> {
    // ...
    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_directory(&put_old)?;
    if let Err(_) = pivot_root(&new_root, &put_old) {
        return Err(Errcode::MountsError(4));
    }
    // ...
}
```

## Unmounting the old root

As we want to achieve isolation with the host system, the "old root" has to be unmounted
so the contained application cannot access to the whole filesystem.

To do this, we create the `unmount_path` and `delete_dir` functions:

``` rust
use nix::mount::{umount2, MntFlags};

pub fn unmount_path(path: &PathBuf) -> Result<(), Errcode>{
    match umount2(path, MntFlags::MNT_DETACH){
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to umount {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountsError(0))
        }
    }
}

use std::fs::remove_dir;

pub fn delete_dir(path: &PathBuf) -> Result<(), Errcode>{
    match remove_dir(path.as_path()){
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to delete directory {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountsError(1))
        }
    }
}
```

And we simply call them at the end of `setmountpoint` function:

``` rust
use nix::unistd::chdir;

pub fn setmountpoint(mount_dir: &PathBuf) -> Result<(), Errcode> {
    // ...
    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));

    // Ensure we are not inside the directory we want to umount
    if let Err(_) = chdir(&PathBuf::from("/")) {
        return Err(Errcode::MountsError(5));
    }
    unmount_path(&old_root)?;
    delete_dir(&old_root)?;
    // ...
}
```

> **Note**: We
umount and delete the `/oldroot.<random_letters>` directory as it was located
inside the `/tmp/crabcan.<random_letters>` directory which became our new `/`.

### The empty cleaning function

You certainly noticed that the `clean_mounts` function is totally useless right now.
The problem is that the parent container doesn't have any clue of where the user-provided
directory got mounted (as it's a randomly generated filename).

The only real problem it causes right now is that all the `/tmp/crabcan.<random_letters>`
directories created still exist after the execution, even if they are empty and unmounted after
the contained process exits.

For the sake of simplicity (or laziness), I let it like this but kept the placeholder for a
cleaning function if it becomes necessary one day.

## Testing

When testing, we can see the new root being located at `/tmp/crabcan.<random_letters>`.
```
[2022-01-04T06:50:25Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2022-01-04T06:50:25Z DEBUG crabcan::container] Linux release: 5.13.0-22-generic
[2022-01-04T06:50:25Z DEBUG crabcan::container] Container sockets: (3, 4)
[2022-01-04T06:50:25Z DEBUG crabcan::container] Creation finished
[2022-01-04T06:50:25Z DEBUG crabcan::container] Container child PID: Some(Pid(324564))
[2022-01-04T06:50:25Z DEBUG crabcan::container] Waiting for child (pid 324564) to finish
[2022-01-04T06:50:25Z DEBUG crabcan::hostname] Container hostname is now blue-man-109
[2022-01-04T06:50:25Z DEBUG crabcan::mounts] Setting mount points ...
[2022-01-04T06:50:25Z DEBUG crabcan::mounts] Mounting temp directory /tmp/crabcan.wYGDJtGIKxZ4
[2022-01-04T06:50:25Z DEBUG crabcan::mounts] Pivoting root
[2022-01-04T06:50:25Z DEBUG crabcan::mounts] Unmounting old root
[2022-01-04T06:50:25Z INFO  crabcan::child] Container set up successfully
[2022-01-04T06:50:25Z INFO  crabcan::child] Starting container with command /bin/bash and args ["/bin/bash"]
[2022-01-04T06:50:25Z DEBUG crabcan::container] Finished, cleaning & exit
[2022-01-04T06:50:25Z DEBUG crabcan::container] Cleaning container
[2022-01-04T06:50:25Z DEBUG crabcan::errors] Exit without any error, returning 0
```

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step10”][code-step10].   
The raw patch to apply on the previous step can be found [here][patch-step10]

[man-sethostname]: https://linux.die.net/man/2/sethostname

[code-step9]: https://github.com/litchipi/crabcan/tree/step9/
[patch-step9]: https://github.com/litchipi/crabcan/compare/step8...step9.diff

[code-step10]: https://github.com/litchipi/crabcan/tree/step10/
[patch-step10]: https://github.com/litchipi/crabcan/compare/step9..step10.diff
