# hippo

Easily manage variants of your configurations and centralize them for portability.

## Usage

    $ hippo snap /etc/nginx/nginx.conf
    Created snapshot `2017-12-22-0255.14` for /etc/nginx/nginx.conf
    
    $ hippo list /etc/nginx/nginx.conf
    Found 2 snapshots for /etc/nginx/nginx.conf
     
    1  (null)                (null)          January 1, 1970 00:00:00
    2* 2017-12-22-0255.14    (no-comment)    December 22, 2017 02:55:14

Now let's say you make changes to your `nginx.conf` file, set it up for serving your
blog. You can save a snapshot after you're done with your changes.

    $ hippo snap /etc/nginx/nginx.conf blog-ready "Setup my blog"
    Created snapshot `blog-ready` for /etc/nginx/nginx.conf
    Old file size: 122B New file size: 180B

Now you can list the snapshots again:

    $ hippo list /etc/ngnx/nginx.conf
    Found 3 snapshots for /etc/nginx/nginx.conf
     
    1  (null)                (null)           January 1, 1970 00:00:00
    2  2017-12-22-0255.14    (no-comment)     December 22, 2017 02:55:14
    3* blog-ready            Setup my blog    December 22, 2017 04:10:07

To see the diff between any two snapshots:

    $ hippo diff /etc/nginx/nginx.conf 2017-12-22-0255.14 blog-ready

To see the diff between a snapshot and the current version of the file:

    $ hippo diff 2017-12-22-0255.14

When using a single parameter, it will always show the diff between the current existing
file on the filesystem, not necessarily the last known diff. If you have edited the file
after taking a snapshot, the same will be reflected in the `list` command.

    $ hippo list /etc/nginx/nginx.conf
    Found 4 snapshots for /etc/nginx/nginx.conf
     
    1  (null)                (null)           January 1, 1970 00:00:00
    2  2017-12-22-0255.14    (no-comment)     December 22, 2017 02:55:14
    3  blog-ready            Setup my blog    December 22, 2017 04:10:07
    4* (live)                (live)           December 22, 2017 04:15:21

Any snapshot name starting and ending with a parenthesis is reserved and is not allowed.

To get a list of all configurations:

    $ hippo list
    /etc/nginx/nginx.conf has 4 snapshots
     
    1  (null)                (null)           January 1, 1970 00:00:00
    2  2017-12-22-0255.14    (no-comment)     December 22, 2017 02:55:14
    3  blog-ready            Setup my blog    December 22, 2017 04:10:07
    4* (live)                (live)           December 22, 2017 04:15:21
     
    /home/rohan/.git/config has 2 snapshots
     
    1  (null)                (null)                 January 1, 1970 00:00:00
    2  base-config           Base config for git    December 22, 2017 02:55:14

Since this list can get very long, if you only want to see the controlled files:

    $ hippo list --summary
    /etc/nginx/nginx.conf has 4 snapshots, currently at `(live)`
    /home/rohan/.git/config has 2 snapshots, currently at `base-config`

To list only specific files:

    $ hippo list --summary /etc/*
    /etx/nginx/nginx.conf has 4 snapshots, currently at `(live)`

Now if you have two nginx configs, one for testing your blog and one for testing your
techies explosion calculator web app, you can swap between the two:

    $ hippo load /etc/nginx/nginx.conf blog-ready
    Loaded snapshot `blog-ready` for /etx/nginx/nginx.conf

If there have been any modifications to the file since the last snapshot, you will get
an error:

    $ hippo load /etc/nginx/nginx.conf techies-calculator
    ERROR: The current snapshot is `(live)`. Create a snapshot for the current file
    before continuing.

You can ask the load command to auto-create a snapshot:

    $ hippo load --auto-snap /etc/nginx/nginx.conf techies-calculator
    Created snapshot `2017-12-22-0511.22` for /etc/nginx/nginx.conf
    Old file size 131B New file size 122B
    Loaded snapshot `techies-calculator` for /etc/nginx/nginx.conf

You can also ask it to skip the check (not recommended) if you want to discard your
changes:

    $ hippo load --discard-live /etc/nginx/nginx.conf techies-calculator
    WARN: The current snapshot is `(live)`. Discarding changes. The last edited version
    is stored in /tmp/49jre9f99g/nginx.conf
 
If temporary storage is not available, or if it is not writable, then the command above
will fail. There is no way around it except for creating a snapshot and then proceeding.

If you want to compress your files:

    $ hippo snap --compress /etc/nginx/nginx.conf production-server

You do not have to specify any decompression required when loading a snapshot, hippo
will figure out the required step on its own.
