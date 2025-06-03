# Commands

This document describes the available commands in the Cat OS Shell, including system, Unix-like, filesystem, and CodeEditor API commands.

---

## System Commands

### ipc

- **Description:** Displays all available IPC (Inter-Process Communication) objects.
- **Usage:** `ipc`
- **Example:**
  ```
  $ ipc
  system/stdio/root
  system/file-system
  ...
  ```

---

## Unix-like Commands

### ls

- **Description:** Lists files and directories in the current working directory.
- **Usage:** `ls`
- **Example:**
  ```
  $ ls
  dir1
  file.txt
  ```

### cd

- **Description:** Changes the current working directory.
- **Usage:** `cd <directory>`
- **Example:**
  ```
  $ cd dir1
  ```

  If no directory is specified, changes to the root directory.

### mkdir

- **Description:** Creates a new directory under the current working directory.
- **Usage:** `mkdir <directory>`
- **Example:**
  ```
  $ mkdir new_folder
  ```

### cat

- **Description:** Displays the contents of a string object (file).
- **Usage:** `cat <file>`
- **Example:**
  ```
  $ cat hello.txt
  Hello, world!
  ```

  If the file is not a string object, an error will be shown.

### clear

- **Description:** Clears the terminal screen.
- **Usage:** `clear`
- **Example:**
  ```
  $ clear
  ```

---

## Filesystem API

### stat

- **Description:** Gets the status (metadata) of an object (file or directory).
- **Usage:** `stat <object>`
- **Example:**
  ```
  $ stat hello.txt
  String
  ```

### get

- **Description:** Displays the specified object, showing its type and content.
- **Usage:** `get <object>`
- **Example:**
  ```
  $ get hello.txt
  String?Hello, world!
  ```

### set

- **Description:** Sets the value of an object. The value must be specified in the correct format (e.g., `String?content`).
- **Usage:** `set <object> <value>`
- **Example:**
  ```
  $ set hello.txt String?Hello, Cat OS!
  ```

---

## CodeEditor API

### load

- **Description:** Loads a file's content into the code editor. Only string files are supported.
- **Usage:** `load <file>`
- **Example:**
  ```
  $ load script.js
  ```

### save

- **Description:** Saves the current content from the code editor to the specified file as a string object.
- **Usage:** `save <file>`
- **Example:**
  ```
  $ save script.js
  ```

---

## Manual Pages

### man

- **Description:** Displays the manual page for a command, or lists all available manuals.
- **Usage:** `man [command]`
- **Example:**
  ```
  $ man ls
  (Displays the manual for 'ls')
  $ man
  (Lists all available manuals)
  ```

---