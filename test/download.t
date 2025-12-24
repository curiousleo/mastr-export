Set up the testing environment.
  $ . "$TESTDIR"/setup.sh

Show the help message.
  $ mastr-export download --help | sed -E 's/ +$//'
  Usage: download [options]...
  Options:
        --if-newer-than IF_NEWER_THAN
                                (input) name of previously downloaded zip file
        --download-dir DOWNLOAD_DIR
                                (output) directory for download
        --help
