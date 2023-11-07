### Acquiring capabilities

The Linux kernel derives a process capabilities from 4 categories of
capabilities:
- **Ambient**: Capabilities that are *granted* to a process, and *can be inherited*
by any child process created.   
In short, **Ambient** capabilities are capabilities **both Permitted and Inheritable**.

- **Permitted**: Capabilities *granted* to a process.   
Any capabilities dropped from the permitted set **can never be reacquired**.

- **Effective**: The actual **applied** permissions of a thread in the given context.

- **Inheritable**: The set of capabilities preserved across an `execve`.   
When starting a new process from a file, adds capabilities from the **Inheritable**
set of the parent process to the **Permitted** set of the child process *only
if these capabilities are not restricted by the file permissions*

- **Bounding**: Used to define the capabilities to drop when performing an `execve`.

To know what kind of capabilities a newly created process have,
the Linux kernel uses these equations:

```
P'(ambient)     = (file is privileged) ? 0 : P(ambient)

P'(permitted)   = (P(inheritable) & F(inheritable)) | (F(permitted) & cap_bset) | P'(ambient)

P'(effective)   = F(effective) ? P'(permitted) : P'(ambient)

P'(inheritable) = P(inheritable) [i.e., unchanged]
```

where:
- `P` denotes the value of a thread capability set before the `execve`
- `P'` denotes the value of a thread capability set after the `execve`
- `F` denotes a file capability set
- `cap_bset` is the value of the capability **Bounding** set.



[linux-user-namespaces-not-secure]: https://medium.com/@ewindisch/linux-user-namespaces-might-not-be-secure-enough-a-k-a-subverting-posix-capabilities-f1c4ae19cad
[hardening-linux-container-pdf]: https://www.nccgroup.com/globalassets/our-research/us/whitepapers/2016/april/ncc_group_understanding_hardening_linux_containers-1-1.pdf
[original-tutorial]: https://blog.lizzie.io/linux-containers-in-500-loc.html
[effect-writing-uidmap]: https://ops.tips/notes/effect-of-writing-to-proc-pid-uid-map/
[uidvsgid]: https://www.cbtnuggets.com/blog/technology/system-admin/linux-file-permission-uid-vs-gid

[code-step11]: https://github.com/litchipi/crabcan/tree/step11/
[patch-step11]: https://github.com/litchipi/crabcan/compare/step10..step11.diff

[code-step12]: https://github.com/litchipi/crabcan/tree/step12/
[patch-step12]: https://github.com/litchipi/crabcan/compare/step11..step12.diff

[man-setgroups]: https://man7.org/linux/man-pages/man2/getgroups.2.html
[man-capabilities]: https://man7.org/linux/man-pages/man7/capabilities.7.html

[cap_dac_override-security]: https://book.hacktricks.xyz/linux-unix/privilege-escalation/linux-capabilities#cap_dac_override
[cap_fowner-security]: https://book.hacktricks.xyz/linux-unix/privilege-escalation/linux-capabilities#cap_fowner
