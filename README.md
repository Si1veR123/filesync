# Filesync - a CLI folder synchronisation app

Filesync uses the current directory's name to download/upload files to google drive.  
From any other device on the same google account, run the download command in a same
named directory to download the files.

## Commands
### up
Upload the local project to google drive

### down
Download the cloud project (with the same named directory) from google drive

### logout
Remove google account from this device (will not remove this app from your google account)

### delete
Delete the project on google drive

### list
List all projects available on google drive

## Example
In `~/projects/test-project` run `filesync up`  
On any other PC logged into the same google drive account, create a folder called `test-project` and run `filesync down`

To remove all projects, delete the filesync app from your google account, or run `filesync delete` in each project directory.

The filesync google drive app isn't officially published and currently only runs in developer mode. This may cause a warning when logging in.
