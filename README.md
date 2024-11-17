# rockdove

filter & redirect github webhooks

## configuration

the following environment variables are required:

- `GITHUB_WEBHOOK_SECRET`: the secret you chose when you created the json webhook
- `DISCORD_WEBHOOK`: the regular discord webhook url
- `DISCORD_BOT_WEBHOOK`: the discord webhook url for bot-authored events
- `DISCORD_USERSTYLES_WEBHOOK`: the discord webhook url for all human events on [catppuccin/userstyles](https://github.com/catppuccin/userstyles).
- `DISCORD_ERROR_WEBHOOK`: the discord webhook url for errors

the following environment variables are optional:

- `PORT`: the port to listen on (default: 3000)

## development

To learn how to forward webhook events to a local instance of rockdove, follow the instructions below:

1. Ensure your `.envrc` has the environment variables listed above in the [configuration](#configuration) section.
2. Compile a release build of rockdove and run it:

   ```shell
   cargo build --release
   ./target/release/rockdove
   ```

3. Install the `gh` cli webhook forward extension:

   ```shell
   gh extension install cli/gh-webhook
   ```

4. Allow `gh cli` to create organisation webhooks on your behalf:

   ```shell
   gh auth refresh -h github.com -s admin:org_hook
   ```

5. Forward the webhook events to your local instance of rockdove:

   ```shell
   gh webhook forward --events='*' --org=catppuccin --url="http://localhost:3000/webhook"
   ```

6. Finally, visit the [GitHub webhook settings](https://github.com/organizations/catppuccin/settings/hooks)
   and paste the `GITHUB_WEBHOOK_SECRET` into the newly created development webhook.
