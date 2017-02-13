# age
Put a file named `.age` in your home directory, containing the birthdates of you and your friends:
```
1987-01-15
1988-05-10 Anne
1984-11-27 Ben
```
Running with the `-a`-flag:
```
You are 30 years old
Anne is 36 years old
Ben is 32 years old
```

Other options:
```
Usage: age [-adhlsw]
Prints your age
        -a      also print ages of other people
        -d      output age in days
        -h      print this help and exit
        -l      long output
        -s      sort in birthday order
        -w      warn if someone has birthday soon
```

I originally wrote this a long time ago in C in order to remind myself of my age.  Re-wrote it in order to learn Rust.
