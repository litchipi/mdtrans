I get why we could think at first that one learns much better when there's no
sugarcoat, and the "real meat" is attacked directly, to *grasp the sense of
what is really going on here*, with no fake assumption.

This is why people still continue to praise learning **C** and despise people who
sticks with Python on their learning, or throw an IP address to a beginner pentester
and ask him to not use all-in-one automated tools to root it.

Of course it depends on what goal you aim for, but allow me to open a vision on
the learning process through script-kiddying and how when well made it can
lead to great results with much less stress, and much more fun.

> This post takes the angle of **pentesting** to explicit my point, however
that is extendable to much larger subjects and many concepts will have their
*metaphoric relative* in other field as well, no matter how far they may be.

# Script kiddying

I feel that you should **never start by learning the theory** of an engine *before
knowing to drive*. However you should never satisfy yourself with only knowing how
to drive.

This is why being a script kiddie is a very important phase of the learning process
of pentesting, because at first you only have to know what button to push where on
what occasion, and *have a sense of why does it work*, but **without** knowing the
technical details of **how does it work**.

Because you keep the hard theory under the hood, you only have the fun and still learns
very well why a thing was vulnerable to such an attack.  

> That is however *not true* if the software used is **too automated**, if you pass
the IP to the software and let it hack the server, you'll learn nothing.

Once you are fluent with all the tools allowing you to "drive", you **should**
start restrict yourself their usage, and replace them with more manual operations,
or re-create the automated tool yourself.

A good way to force you into this is to prepare yourself to pass the **OSCP** certification,
for which the major automated software are forbidden during the exam
(sqlmap, metasploit, burp, ...)

# Why re-write an existing tool ?

> Real hackers make their own tools

That's something I've always been told by any of my friends when talking about
pentesting. **I doubt** that any professionnal pentester would not use
such a *complete* tool like `sqlmap` to automate a SQL injection (not my favourite
part of hacking I have to admit), however this statement is *not entirely false*.

Any hacker will have its own unique way of testing a target, because of the tools,
the angle, the vision of the technology he has. Anybody want's to be able to use
a tool that is perfectly suited for him, think of it as a carpenter having a
hammer handle specially made for his palm and fingers, you can do without, but damn
it feels good !

## Copy the tools you use

Even if we like a tool very much, there's always something about it that doesn't
totally satisfies us.  
Personally:
- I think *Burp* is too much click-based, and I'd prefer having a CLI/TUI interface.
- I dislike *metasploit*, I think most of it is sugar, the real usefull tool being
`msfvenom` only.
- I think *sqlmap* is generating a ton of traffic and hits way too hard the target.

So once I know what I don't like in the tools I use, the simplest step is to remove
them entirely from the workflow.  
If you need one of them for a particular operation, you'll have to re-build that
operation yourself, with a good ol' bash / Python script.

Copy the usefull bit of the software you want to replace, also the things you really
liked in it, but feel free to please yourself on how to use it.  
You want a *GUI* ? Build one !
Some *Lua* scripts to be run between steps of a process ? Go for it !

It's your tool, for your preferences, and there are **no good or bad answers** on
the design choices, as long as it gets the job done.

Don't try to create a all-in-one pentesting toolbox, instead create a custom tool
for each operation (and its close derivatives), and try to re-use your code as much
as possible.

> This can be also applied to any art.  
> Copying artists that inspires you, or smart technics to achieve a result, but only
> take what you like in it, appropriate the copied art to your vision and preferences,
> and mix it with your other inspirations and tools.

> There's about as much artists who never copied the art of someone as there are
> programmers who never copied a StackOverflow answer.  
> None

## Make it modular and pretty

It doesn't seam like the top 1 priority, but create a tool you find visually appealing.
You need to *be proud* of what you created, the whole learning process for pentesting
or anything else is very mental-based, so you need to be proud of yourself

Also, you don't want a piece of software that can only be used for a single thing,
on a single context. You want to be able to use a code you wrote as a building brick
for any other tool you may need.

For example, I recently created a HTTP Proxy in Python after reading the great
[Black Hat Python][blackhatpython] book, you can check the code [here][httpproxy],
and it allows me to create custom and complex intercept / replace operations on
the data very simply:

``` python
from tcp_proxy import HttpProxy, Arguments

class Handler:
    def __init__(self):
        self.emails = list()

    def handle_response(self, header, payload):
        self.emails.extend(catch_all_emails(payload))
        return header, payload

    def handle_request(self, header, payload):
        return header, payload.replace(b"TATA", b"toto")

def catch_all_emails(data):
    return []   # Something with regex

if __name__ == '__main__':
    args = Arguments().get_args()
    proxy = HttpProxy(Handler(), args.port)
    proxy.loop()
```

Imagine when submiting a HTTP form, the website expects you to include in the request
the value of a token that is randomly generated for each page reloading.
(Situation encountered during a CTF)  
Here I simply have to grab the value of the token in `handle_response` and store
it in a class variable, then pass the value to any HTTP form request intercepted
in the `handle_request` function.

Everything I describe here can be easily done in a few clicks with Burp and a lot
of walkthrough always use these tools (they won't ask you to re-write a whole proxy),
but now it's done *my way*, with code only, no clicks, and tailored for the target
in front of me.

You need the right lockpicks to pick the right lock, and even if a pick-gun will
do, you'll always learn more about the lock by manually locking it.

That's also why I created [fart][fartcode], a packet interception / edition, which
brings nothing more than what Burp has to offer, only that it's my dirty code,
and I'm damn proud of it **:D**

# Learning by mimicking

I won't make the metaphor with the babies learning process, however I'd like to resume
the learning path using the "Copy it until you make it" process:

- Use script-kiddie technics to root your first CTF box
- Become very fluent at rooting CTF boxes using your favourite set of tools
- Replace your tools one-by-one with custom Python / bash scripts to have custom
ones and learn more about the underlying technics
- Finally, have a complete set of custom tools covering most of the operations
you need to perform to root a CTF box, can be adapted to your specific needs,
and you know the processes that make these operations work, because you made them.

Sounds to me like a great plan to really get a firm grasp on what is going on,
while having fun, and advancing step by step, not to mention improvements in
writing good modular code, network programming, shellcode generation, etc...  
Also if you want to step-up your tooling, you can write them in another langage,
like Go (check the book [Black hat Go][bhgo]) or even Rust (check the book
[Black hat Rust][bhrust]).

I'd say the Offensive Security motto **"Try harder"** should be replaced by
**"Try easier, spice up later"**

[blackhatpython]: https://nostarch.com/black-hat-python2E
[httpproxy]: https://github.com/litchipi/http_proxy
[fartcode]: https://github.com/litchipi/fart
[bhgo]: https://nostarch.com/blackhatgo
[bhrust]: https://kerkour.com/black-hat-rust
