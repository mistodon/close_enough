close_enough
===

[![Crates.io](https://img.shields.io/crates/v/close_enough.svg)](https://crates.io/crates/close_enough)
[![Docs.rs](https://docs.rs/close_enough/badge.svg)](https://docs.rs/close_enough/0.5.0/close_enough/)

Simple fuzzy-searching function and command line tool.


Installation
---

`cargo install close_enough`


### Installing the `ce` command

This is a `cd`-like command for fuzzily changing directories. See examples further down.

1.  Install `close_enough` as above
2.  Run `cle -ce-script bash > ce.sh` to generate the shell script
    - If you also want to use the `hop` command, you should use: `cle -ce-script bash --with-hop > ce.sh`
3.  Source `ce.sh` in your `.bashrc`, `.profile`, or similar

### Installing the `hop` command

This allows you to track recently used folders (you can define wrappers around `cd` or `ce` to do this automatically) and hop directly to them with a fuzzy match.

1.  Install `close_enough` as above
2.  Run `cle -hop-script bash > hop.sh` to generate the shell script
3.  Source `hop.sh` in your `.bashrc` etc.


Usage
---

### cle

```sh
~$ cle duck --inputs blue_and_gold_macaw duck_billed_platypus angry_dog
> duck_billed_platypus

~$ cle dbp --inputs blue_and_gold_macaw duck_billed_platypus angry_dog
> duck_billed_platypus
```

```sh
~$ ls
> my_file.txt  their_file.txt  your_file.txt
~$ ls | cle my
> my_file.txt
```


### ce

```sh
~$ ce my lo dir pa
~/my/long/directory/path$
```

```sh
~/my/long/directory/path$ ce ..
~/my/long/directory$
```

```sh
~/my/long/directory/path$ ce ..3
~/my$
```

```sh
~/my/long/directory/path$ ce ..my other dir pa
~/my/other/directory/path$
```

```sh
~$ ce / u lo sh
/usr/local/share$ ce ~
~$
```


### hop

```sh
~$ ce my dir 1
~/my/directories/d1$ ce .. 2
~/my/directories/d2$ cd
~$ # If you used --with-hop then hop will have tracked the d1 and d2 dirs
~$ hop to d1
~/my/directories/d1$ hop to d2
~/my/directories/d2$
```
