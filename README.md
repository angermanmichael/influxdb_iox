
### Branch History

### nu84b2f

* start work on fixing the errors so nushell never crashes

### nu84b2e

* get the latest iox code and merge it on to this branch

### nu84b2d

* all of the nushell commands are now working on this branch

### nu84b2c

Now that *nushell* is up and running and we have the *nuclient* infrastructure
in place lets start adding in our first command to talk to iox.

### nu84b2b

*nu-command* is now compiling, now lets integrate in the nushell binary so
we can bring up nushell and start running the iox commands inside nushell.

### nu84b2a

add back in the needed crates for nu-command and remove the files I am not
yet working on including:

* namespace.rs
* sql.rs
* write.rs
* writefile.rs

#### nu84b2

some of the original commands added in nu84b1 have been deleted so I can
get just one command to compile and working.  stay tuned for what command
this will be.

#### nu84b1

is the iox commands integrated in and does not compile

#### nu84a

is the original branch with the nu-command code integrated in prior
to adding in the iox commands... This code compiles

stay tuned !
