# Certman
A module for managing certificates.

# Reference
## Root
Namespace: `certman/root`

Responsible for managing root certificate which
stays in the beginning of any certificate chain.

### Commands
#### `show`
Full command: `certman/root/show`

Shows a root certificate or prints a message that
there is no such.

#### `generate`
Full command: `certman/root/generate`

Arguments:
* `name` a name of ceritifcate to generate

**Example**

```
mway> certman/root/generate name=example.com
Certificate generation successful
```

#### `export`
Full command: `certman/root/export`

Exports root certificate to a given file.

Arguments:
* `file` file name to save root certificate to

**Example**

```
mway> certman/root/export file=root.pqcert
Export successful
```


#### `import`
Full command: `certman/root/import`

Imports root certificate from a given file.

Arguments:
* `file` file to load certificate from


**Example**

```
mway> certman/root/import file=root.pqcert
Loaded certificate successfully
Registere certificate in service
```