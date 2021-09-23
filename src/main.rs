use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};
use std::env;

use serenity::{
  async_trait,
  model::{channel::Message, gateway::Ready},
  prelude::*,
};

struct Handler;

#[derive(Serialize, Deserialize, Debug)]
struct GiphyOriginalImage {
  url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GiphyImages {
  original: GiphyOriginalImage,
}

#[derive(Serialize, Deserialize, Debug)]
struct GiphyData {
  images: GiphyImages,
}

#[derive(Serialize, Deserialize, Debug)]
struct GiphyResponse {
  data: Vec<GiphyData>,
}

async fn get_selfie() -> Result<String, Box<dyn std::error::Error>> {
  let giphy_token = env::var("GIPHY_TOKEN").expect("Expected a giphy token in the environment");
  let resp = reqwest::get(
    format!("https://api.giphy.com/v1/gifs/search?api_key={}&q=squirtle&rating=g&lang=en", giphy_token)
  ).await?.json::<GiphyResponse>().await?;
  Ok(resp.data.choose(&mut rand::thread_rng()).unwrap().images.original.url.to_owned())
}

#[derive(Serialize, Deserialize, Debug)]
struct MoveDetails {
  name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
  r#move: MoveDetails,
}

#[derive(Serialize, Deserialize, Debug)]
struct PokeAPIResponse {
  moves: Vec<Move>,
}

async fn get_moves() -> Result<String, Box<dyn std::error::Error>> {
  let resp = reqwest::get(
    "https://pokeapi.co/api/v2/pokemon/squirtle"
  ).await?.json::<PokeAPIResponse>().await?;
  Ok(resp.moves.choose(&mut rand::thread_rng()).unwrap().r#move.name.to_owned())
}

#[async_trait]
impl EventHandler for Handler {
  // Set a handler to be called on the `ready` event. This is called when a
  // shard is booted, and a READY payload is sent by Discord. This payload
  // contains data like the current user's guild Ids, current user data,
  // private channels, and more.
  //
  // In this case, just print what the current user's username is.
  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
  }

  // Set a handler for the `message` event - so that whenever a new message
  // is received - the closure (or function) passed will be called.
  //
  // Event handlers are dispatched through a threadpool, and so multiple
  // events can be dispatched simultaneously.
  async fn message(&self, ctx: Context, msg: Message) {
    match ctx.http.get_current_application_info().await {
      Ok(info) => {
        if msg.content == "!ping" {
          // Sending a message can fail, due to a network error, an
          // authentication error, or lack of permissions to post in the
          // channel, so log to stdout when some error happens, with a
          // description of it.
          if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            println!("Error sending message: {:?}", why);
          }
        } else if msg.mentions_user_id(info.id) {
          let cries = vec!["Squirtle!", "Squirtle squirtle squirtle", "SQUIRTLE SQUIRTLE!"];
          let cry = cries.choose(&mut rand::thread_rng()).unwrap();
          if msg.content.contains("take a selfie") {
            let selfie = get_selfie().await.unwrap();
            if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
              m.content(&cry);
              m.embed(|e| {
                e.image(&selfie);
                e
              });
              m
            }).await {
              println!("Error sending message: {:?}", why);
            }
          } else if msg.content.contains("attack") {
            let attack_move = get_moves().await.unwrap();
            if let Err(why) = msg.channel_id.say(&ctx.http, format!(
              "Squirtle use {}. It's super effective.", attack_move
            )).await {
              println!("Error sending message: {:?}", why);
            }
          } else {
            if let Err(why) = msg.channel_id.say(&ctx.http, &cry).await {
              println!("Error sending message: {:?}", why);
            }
          }
        }
      },
      Err(why) => println!("Error getting current user: {:?}", why),
    }
  }
}

#[tokio::main]
async fn main() {
  // Configure the client with your Discord bot token in the environment.
  let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
  
  // Create a new instance of the Client, logging in as a bot. This will
  // automatically prepend your bot token with "Bot ", which is a requirement
  // by Discord for bot users.
  let mut client =
  Client::builder(&token).event_handler(Handler).await.expect("Err creating client");
  
  // Finally, start a single shard, and start listening to events.
  //
  // Shards will automatically attempt to reconnect, and will perform
  // exponential backoff until it reconnects.
  if let Err(why) = client.start().await {
    println!("Client error: {:?}", why);
  }
}
