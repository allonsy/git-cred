# git-cred
Simple Encrypted git credential management

## Features
* Securely store secret keys and files in your repo
* Uses GPG to secure files
* Allows flexible user management
* Subfolders allow granular control over access controls to secrets
* Automatically looks up keys in github for easy key sharing

## Example Workflow
Let's say I'm in my git repo and I want to create a store:

`> git cred init allonsy friend@email.com`

The above command automatically creates a new credential store, it will encrypt all files for `allonsy` and `friend@email.com`.

Then, to encrypt a secret api key, I'll run:

`> git cred encrypt api_key "super_secret_key"`

This will automatically encrypt the api key in the credential store. In order to find the key for allonsy, it will make a call to github's api and automatically download the GPG keys I've stored there. Then, it will use the first available valid key. To get the key for `friend@email.com`, it will check my local gpg keyring as use the key for that identity. 
I can then run `git add`, `git commit`, and `git push` on the new `.credential_store` directory to push this key, safe in the knowledge that only myself and `friend@gmail.com` can decrypt it.

To see the key, I run:

`> git cred decrypt api_key`

to see the string: `super_secret_key` output to the console. The decrypt command can of course be used in scripts to set environment variables or create temp files.

Now let's suppose I want to store keys that I don't want `friend@email.com` to see. I can store these keys in a subfolder. I create this subfolder like so:

`> git cred init -f special allonsy` this creates a `special` subfolder and it will only encrypt files for user: `allonsy`. 
Now, if I run:

`> git cred encrypt special/api_key_2 "super_super_secret_key"`

I'll be able to decrypt it by running:

`> git cred decrypt special/api_key_2`

but `friend@email.com` will error out when trying to decrypt that file. 

Now, let's suppose that we add another person to our team: `sidekick@email.com` who is my sidekick. I want to give him the same access as `friend@gmai.com`. I add this person like so:

`> git init allonsy friend@email.com sidekick@email.com`

This will set the gpgs of the root folder to those 3 gpg ids. It will automatically reencrypt our credential store with those 3 gpg ids. It won't touch the subfolder `special` since I didn't ask it to. Therefore, my secrets in `special` are still safe.

Now, let's say that I have the public key for `sidekick@email.com` in my local gpg keyring by sidekick isn't on github. Therefore, when `friend@email.com` pulls my changes and tries to encrypt something new, `friend` won't have `sidekick`'s public key. To solve this, I can run:

`> git cred save-key sidekick@email.com`

This will store the public key into the credential store. Then, when `friend` pulls the repo, they have the key and can run `git cred encrypt` as normal (`git cred` will read the key from the store and auto-import it into `friend`'s local gpg keyring (the key will remain untrusted until `friend` manually verifies its fingerprint))

## Usage

`git cred (subcommand)`

where subcommand is one of the following:
* `init`
* `encrypt`
* `decrypt`
* `reencrypt`
* `save-key`
* `help`

### Init
Init a credential store or subfolder within the store

Usage: `git cred init [-f folder_name] [gpg_ids...]`

`-f folder_name`: instead of intializing the root folder of the credential store, intialize a subfolder.

`[gpg_ids...]`:   a space separated list of gpg ids to use for encryption. These gpgs will be specific to that folder (or root folder if none specified. These can be emails, gpg key ids, or github usernames if no gpgs are provided, the field of user.email in your git config is used.

Notes:
* The default location for the credential store is `.credential_store` in the root of your repo. To change this, run the following command: `git config creds.location <location>` where `location` is a path relative to the root of the git repo.
* If the folder you provide (or the root folder) already exists, the existing gpg ids will be overwritten and the target folder will be automatically reencrypted with the provided gpg_ids.
* This command isn't strictly necessary, all the other calls will call init automatically. However, if you want to use anything other than your default email, you need to call this with desired gpg_ids.

Examples:
* `git cred init`
* `git cred init -f foo/bar`
* `git cred init username1`
* `git cred init username1 email@email.com`
* `git cred init -f foo username1 email@email.com`

### Encrypt
Encrypt a file or string in the store

Usage: `git cred encrypt <path_to_encrypt> (-f <file_name> | <string_to_encrypt>)`

`path_to_encrypt`:  the location in the credential store you want to encrypt to (e.g. foo/bar)

`-f file_name`: you may provide a file that will be encrypted via this flag 

`string_to_encrypt`:    instead of providing a file, you can simply write the string to encrypt as a single command line argument

Examples:
* `git cred encrypt foo hello`
* `git cred encrypt foo \"hello, world!\"`
* `git cred encrypt foo/bar hello`
* `git cred encrypt foo -f secret.txt`

### Decrypt
Decrypt a file in the store

Usage: git `cred decrypt <file_path>`

`file_path`:  the file path to decrypt. The decrypted string will be output to standard out

Examples:
* `git cred decrypt foo`
* `git cred decrypt foo/bar`

### Reencrypt
Reencrypt your credential store based on the gpg ids already present in the store.
To reencrypt with different gpg ids, use `git cred init` instead.

Usage:  `git cred reencrypt`

Notes:
* This command is mainly used when the .gpg_id files are manually edited as changes via `git cred init` automatically reencrypt the credential store.

Examples:
* `git cred reencrypt`

### save-key
Store a public key from your gpg keyring/github into the repo

Usage: `git cred save-key <uid> [keyfile]`

`uid`:  Either a github username, email, or gpg key id. The key for that uid will be looked up and the public key will be inserted into the repo for other users to encrypt with.

`[keyfile]`:  instead of looking up the public key for that uid, the public key in the provided keyfile will be saved instead. All further encryption for that uid will use that key instead. You may remove the keyfile after this command completes

Notes:
* This function can be useful to set the desired key if a given email has more than one public key associated with it.

Examples:
* `git cred save-key username1`
* `git cred save-key AAABBBCCC`
* `git cred save-key email@email.com`
* `git cred save-key email@email.com /path/to/keyfile.asc`

### help
Bring up usage and help text to the console

Usage: `git cred help [subcommand]`

`subcommand`:   Bring up help for a specific subcommand rather than the general help page

Examples:
* `git cred help`
* `git cred help init`