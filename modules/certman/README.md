# Certman

A module for managing certificates.

# Reference

## Certificates

The key purposes of certificate is identification of peers and establishing secure communications.

The certificates consists of:

* Serial number
* Public key
* Secret key
* Parent serial number(a serial number of certificate which was used to sign this one, if applicable)
* Flags showing purpose of certificate
* String name

### Flags

| Flag short name | Description                                           |
|-----------------|-------------------------------------------------------|
| client-cert     | A certificate for use by clients machine              |
| no-read         | A certificate can not read configurations or statuses |
| no-write        | A certificate can not write or execute commands       |
| server-cert     | A certificate can be used for server                  |
| sign-certs      | A certificate sign other certificates                 |
| sign-messages   | A certificate can sign messages                       |
| user-cert       | A certificate can be used by CLI to connect to server |

## Root command namespace

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

## Signing command namespace
A namespace for managing signing certificates. 

#### `export`

Full command: `certman/signing/export`

Exports root certificate to a given file.

Arguments:

* `file` file name to save root certificate to

**Example**

```
mway> certman/signing/export serial=1 file=root.pqcert
Export successful
```

#### `import`

Full command: `certman/root/import`

Imports root certificate from a given file.

Arguments:

* `file` file to load certificate from

**Example**

```
mway> certman/signing/import file=mycert.pqcert
Loaded certificate successfully
Registered certificate in service
```

#### `generate`

Full command: `certman/signing/generate`

Arguments:

* `name` a name of ceritifcate to generate
* `parent-serial` a serial number of parent certificate which would be use to sign new one
* `flags` a flags to set

**Example**

```
mway> certman/signing/generate name=example.com parent-serial=0 flags=server-cert,sign-certs,sign-messages
Certificate generation successful
Certificate signed with parent serial 0
```