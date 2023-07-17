# AI Editor

The back end of Arguflow AI Coach and Arguflow vault

This repository utilizes [actix-web](https://actix.rs), a [Rust](https://www.rust-lang.org) language framework

## How to contribute

1. Fork the repository and clone it to your local machine
2. Create a new branch with a descriptive name: git checkout -b your-branch-name
3. Make your changes to the README file. Please ensure that your changes are relevant and add value to the project
4. Test your changes locally to ensure that they do not break anything
5. Commit your changes with a descriptive commit message: git commit -m "Add descriptive commit message here"
6. Push your changes to your forked repository: git push origin your-branch-name
7. Open a pull request to the main repository and describe your changes in the PR description

## Storing environment variables in .env file

Create a .env file in the root directory of the project. This .env file will require the following url's and API keys

```
DATABASE_URL=postgresql://postgres:password@localhost:5432/ai_editor
REDIS_URL=redis://127.0.0.1:6379/
SENDGRID_API_KEY=SG.XXXXXXXXXXXXXXXXXXxx
OPENAI_API_KEY=sk-XXXXXXXXXXXXXXXXXXxx
DOMAIN=localhost
ALLOWED_ORIGIN=http://localhost:3000
STRIPE_API_SECRET_KEY=sk_test_XXXXXXXXXXXXXXXXXXxx
S3_ACCESS_KEY=XXXXXXXXXXXXXXXXXXxx
S3_SECRET_KEY=XXXXXXXXXXXXXXXXXXxx
S3_BUCKET=XXXXXXXXXXXXXXXXXXxx
S3_ENDPOINT=XXXXXXXXXXXXXXXXXXxx
```

## APT packages needed

`chromium-browser`
`openssl`

## Setting Up Local S3

1. `sudo docker compose up s3`
2. Go to the MinIO dashboard at [http://127.0.0.1:42625](http://127.0.0.1:42625)
3. Sign in with `rootuser` and `rootpassword`
4. Go to [http://127.0.0.1:42625/identity/users](http://127.0.0.1:42625/identity/users)
5. Set `S3_ENDPOINT` to `http://127.0.0.1:9000`
6. Click "Create User"
7. Set the credentials to "s3user" and "s3password"
8. Assign the user roles for access
9. Click the user and navigate to "Service Accounts"
10. Click "Create Access Key" and save the created keys to your .env under `S3_ACCESS_KEY` and `S3_SECRET_KEY` respectively
11. Click "Buckets" and then "Create Bucket"
12. Create a bucket named `ai-editor` and set `S3_BUCKET` in the env to `ai-editor`

## Preparing for file uploads and conversions

1. `sudo apt install pandoc`
2. `mkdir tmp` from inside repository folder

If you want to test things outside of the minio dashboard and Tokio server, then setup AWS CLI using [this guide from minio](https://min.io/docs/minio/linux/integrations/aws-cli-with-minio.html) with the region set to `""`

- You can install it with `sudo apt install awscli` assuming you are using the apt package manager

## Running the test suite

This section refers to the jest testing suite found in the `jest-tests` folder of this repository

1. Set the variables found in `.env.dist` in the `.env` file of the testing suite
2. `cd` into the testing suite folder
3. Run `yarn`
4. Run `yarn test`

## Resetting the db's

1. `sudo docker compose stop qdrant-database`
2. `sudo docker compose rm -f qdrant-database`
3. `sudo docker volume rm ai-editor_qdrant_data`
4. `diesel db reset`

## How to debug diesel by getting the exact generated SQL

diesel::debug*query::<diesel::pg::Pg, *>(&query);

## How to set up the python verification script

1. `virtualenv venv`
2. `source venv/bin/activate`
3. `pip install -r ./python-scripts/requirements.txt`


## How to get Rust debug level logs 

Run `export RUST_LOG=debug`
