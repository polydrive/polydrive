# CLI Reference

Below you'll find the documentation of `polydrive` commands.

You can also use the `--help` arguments to get the detailed help for each command.

```bash
$ polydrive --help
# or
$ polydrive COMMAND --help
```

## `list`

List the files synchronized in the system 

```bash
$ polydrive list --daemon --watch /tmp
```

Options :

- `-d, --daemon` : If set, the client will be act as a daemon
- `--watch <FILES>` : A list of files or directories to watch.
- `-c, --config <PATH>`: The config file used by the client.
- `-v, --verbose` : Display debug logs
- `-vv, --verbose --verbose` : Display trace and debug logs
- `-vvv, --verbose --verbose --verbose` : Display trace, debug and info logs