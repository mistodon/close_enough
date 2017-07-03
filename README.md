close_enough
===

Simple fuzzy-searching function and command line tool.

Installation
---

`cargo install close_enough`

### Installing the `ce` command

1.  Install `close_enough` as above
2.  Run `cle -gen-script ce > ce.sh` to generate the shell script
3.  Source `ce.sh` in your `.bash_rc`, `.profile`, or similar

Usage
---

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

To do
---

1.  Make `ce` ignore trailing slashes (or tab completion can get in the way)
2.  Make `ce --help` format properly
3.  Make `ce` understand a leading slash
4.  Make `ce` understand the tilde for home directory
