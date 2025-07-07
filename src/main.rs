use std::{
    fs::{read_dir, remove_file, write},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use dotenv_codegen::dotenv;
use poise::{
    ApplicationContext,
    reply::CreateReply,
    serenity_prelude::{
        self as serenity, ButtonStyle, ComponentInteractionCollector, CreateActionRow,
        CreateAttachment, CreateButton, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    },
};

#[tokio::main]
async fn main() {
    let token = dotenv!("DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![calcagebra()],

            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;

/// Execute code with calcagebra
#[poise::command(slash_command)]
async fn calcagebra(
    ctx: ApplicationContext<'_, Data, Error>,
    #[description = "Expression to run [there is no need to enclose it in print()]"] code: Option<
        String,
    >,
) -> Result<(), Error> {
    let contents = if code.is_some() {
        format!("print({})", code.unwrap())
    } else {
        use poise::Modal as _;

        #[derive(poise::Modal)]
        struct Modal {
            #[placeholder = "print(0)"]
            #[paragraph]
            code: String,
        }

        let data = Modal::execute(ctx).await?;

        data.unwrap().code
    };

    let start = SystemTime::now();
    let time = start.duration_since(UNIX_EPOCH).unwrap().as_nanos();

    let file = format!("temp{time}",);

    write(&file, &contents).unwrap();

    let result = Command::new("./calcagebra.exe")
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();

    let response = String::from_utf8(if !result.stdout.is_empty() {
        result.stdout
    } else {
        result.stderr
    })
    .unwrap();

    remove_file(&file).unwrap();

    let mut builder = CreateReply {
        ..Default::default()
    }
    .embed(
        CreateEmbed::new()
            .field("Input", &format!("```rs\n{contents}\n```"), false)
            .field("Output", &format!("```\n{response}\n```"), false)
            .footer(CreateEmbedFooter::new("Images attached somewhere"))
            .author(CreateEmbedAuthor::new(
                result.status.code().unwrap().to_string(),
            )),
    )
    .components(vec![CreateActionRow::Buttons(vec![
        CreateButton::new("nonce")
            .style(ButtonStyle::Danger)
            .label("Delete")
            .emoji('🗑'),
    ])]);

    for file in read_dir("./").unwrap().filter(|f| {
        f.as_ref()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .starts_with("graph-output-")
    }) {
        let file = file.unwrap().file_name().into_string().unwrap();
        builder = builder.attachment(CreateAttachment::path(&file).await?);
        remove_file(&file).unwrap();
    }

    ctx.send(builder).await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with("nonce"))
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        press.message.delete(ctx).await?;
    }

    Ok(())
}
