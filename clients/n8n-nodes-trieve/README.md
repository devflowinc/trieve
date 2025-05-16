# n8n-nodes-trieve

This contains the n8n nodes for using Trieve with n8n.

## Local development

You need the following installed on your development machine:

- Install n8n with:
  ```
  npm install n8n -g
  ```
- Run `n8n` to start n8n for the first time
- Loin to [http://localhost:5678](http://localhost:5678) and complete onboarding steps and this should create a folder in `~/.n8n.`
- To allow the node to be loaded locally run:

   ```
   mkdir ~/.n8n/custom/
   ln -s "$(pwd)/n8n-nodes-trieve ~/.n8n/custom/n8n-nodes-trieve
   ```

- Run `npm run build` inside the `n8n-nodes-trieve` directory
- Start `n8n` again, you should see the node in the sidebar.
- 
