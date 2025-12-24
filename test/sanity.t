Set up the testing environment.
  $ . "$TESTDIR"/setup.sh

This should print 'hello'.
  $ echo hello
  hello

List the environment variables that are not filtered out by the environment
setup.
  $ env | cut -d= -f1 | sort
  MASTR_EXPORT_TAG
  PATH
  PWD
  TESTDIR
  _
