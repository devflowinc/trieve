# AI Editor

The back end of Arguflow AI is written in ['actix-web'](https://actix.rs), a ['Rust'](https://www.rust-lang.org) language framework. 

## How to contribute

1. Fork the repository and clone it to your local machine.
2. Create a new branch with a descriptive name: git checkout -b your-branch-name
3. Make your changes to the README file. Please ensure that your changes are relevant and add value to the project.
4. Test your changes locally to ensure that they do not break anything.
5. Commit your changes with a descriptive commit message: git commit -m "Add descriptive commit message here"
6. Push your changes to your forked repository: git push origin your-branch-name
7. Open a pull request to the main repository and describe your changes in the PR description.

## Storing environment variables in .env file

Create a .env file in the root directory of the project. This .env file will require the following url's and API keys.

'''
DATABASE_URL=postgresql://postgres:password@localhost:5432/ai_editor
REDIS_URL=redis://127.0.0.1:6379/
SENDGRID_API_KEY=SG.XXXXXXXXXXXXXXXXXXxx
OPENAI_API_KEY=sk-XXXXXXXXXXXXXXXXXXxx
DOMAIN=localhost
ALLOWED_ORIGIN=http://localhost:3000
STRIPE_API_SECRET_KEY=sk_test_XXXXXXXXXXXXXXXXXXxx
'''
