# Object-Oriented File System

## Overview

This file system is designed with an object-oriented architecture, allowing flexible and extensible management of files and directories. Unlike traditional file systems that only store byte streams, this system can directly store typed values such as numbers, strings, booleans, and more.

## Key Features

- **Object-Oriented Structure**: Every file system entity (file, directory, or intrinsic value) is represented as an object implementing a common interface.
- **Typed Value Storage**: Supports direct storage of intrinsic types (integers, floats, strings, booleans, bytes, null) as file system objects.
- **Hierarchical Directories**: Directories can contain other directories or files, forming a tree structure.
- **Extensible**: New object types can be added by implementing the required interfaces.

## File System Objects

There are two main categories of objects:

### 1. CompoundFSObj

Represents directories or containers that can have children (other files or directories).

- **Fields**:
  - `parent`: Reference to the parent directory (if any).
  - `children`: Map of child names to their corresponding objects.
- **Methods**:
  - `list()`: Lists the names of all children.
  - `get_obj(name)`: Retrieves a child object by name.
  - `add_child(name, obj)`: Adds a new child object.

### 2. IntrinsicFSObj

Represents intrinsic values stored directly in the file system.

- **Supported Types**:
  - Integer (`Int`)
  - String (`String`)
  - Boolean (`Boolean`)
  - Float (`Float`)
  - Double (`Double`)
  - Bytes (`Bytes`)
  - Null (`Null`)
- **Methods**:
  - Implements the same interface as other objects, but does not support children.

## Example Structure

```
/
├── usr/
│   ├── mime/
│   ├── app/
│   └── ref/
├── mnt/
└── workspace/
```

Each directory (e.g., `usr`, `mnt`, `workspace`) is a `CompoundFSObj`. Intrinsic values can be stored as files within any directory.

## API Overview

The file system exposes the following main operations:

- `list(path)`: List contents of a directory.
- `stat(path)`: Get metadata about a file or directory.
- `get(path)`: Retrieve the value or object at a given path.
- `set(path, value)`: Store a value at a given path.
- `mkdir(path, name)`: Create a new directory.

## Usage Notes

- Paths are resolved from the root (`/`). Relative paths are not currently supported.
- When storing a value, its type is preserved and can be retrieved without manual serialization.
- The system is designed to be extended with new object types as needed.

## Example: Storing and Retrieving a Number

```typescript
await fs.set_raw("/workspace/answer", "Integer?42");
const [kind, value] = await fs.get("/workspace/answer");
// kind == "Integer", value == "42"
```

## Error Handling

Common errors include:

- `InvalidCommandFormat`
- `UnsupportedMethod`
- `UnknownPath`
- `UnknownError`

These are returned as strings and should be handled appropriately in client code.

---
