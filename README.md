# rockdove

filter & redirect github webhooks

## configuration

the following environment variables are required:

- `GITHUB_WEBHOOK_SECRET`: the secret you chose when you created the json webhook
- `DISCORD_WEBHOOK`: the regular discord webhook url
- `DISCORD_BOT_WEBHOOK`: the discord webhook url for bot-authored events

the following environment variables are optional:

- `PORT`: the port to listen on (default: 3000)
