Raiden
======

Raiden is a tool that splits a given file to N "disk" files according to the RAID 5 algorithm. It can merge the original file with N - 1 "disks".

Why is it useful?
-----------------
It's not. I wrote it for learning purposes.

Usage
-----

    raiden split some_file 5

will split `some_file` to 5 parts called `some_file_N`. You can delete one of the parts and then run:

    raiden merge some_file 5

which should be able to restore the original file as `res__some_file`