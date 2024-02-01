# ls-proxy

A language server proxy. The idea is to have a generic proxy to talk to a LS that runs inside a container, so you don't need to install all the LS on your local machine but can use images instead.

I'm writing this to use with NeoVim.

## TODO

- add --tee cli with clap
- why is podman still running after closing nvim?
