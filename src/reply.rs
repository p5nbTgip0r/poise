//! Infrastructure for replying, i.e. sending a message in a command context
//!
//! This file dispatches to command-type specific reply code, which is in the prefix and slash
//! modules

use crate::serenity_prelude as serenity;

/// Message builder that abstracts over prefix and application command responses
#[derive(Default, Clone)]
pub struct CreateReply<'att> {
    /// Message content.
    pub content: Option<String>,
    /// Embeds, if present.
    pub embeds: Vec<serenity::CreateEmbed>,
    /// Message attachments.
    pub attachments: Vec<serenity::AttachmentType<'att>>,
    /// Whether the message is ephemeral (only has an effect in application commands)
    pub ephemeral: bool,
    /// Message components, that is, buttons and select menus.
    pub components: Option<serenity::CreateComponents>,
    /// The allowed mentions for the message.
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// The reference message this message is a reply to.
    pub reference_message: Option<serenity::MessageReference>,
}

impl<'att> CreateReply<'att> {
    /// Set the content of the message.
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed to the message.
    ///
    /// Existing embeds are kept.
    pub fn embed(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateEmbed) -> &mut serenity::CreateEmbed,
    ) -> &mut Self {
        let mut embed = serenity::CreateEmbed::default();
        f(&mut embed);
        self.embeds.push(embed);
        self
    }

    /// Set components (buttons and select menus) for this message.
    ///
    /// Any previously set components will be overwritten.
    pub fn components(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateComponents) -> &mut serenity::CreateComponents,
    ) -> &mut Self {
        let mut components = serenity::CreateComponents::default();
        f(&mut components);
        self.components = Some(components);
        self
    }

    /// Add an attachment.
    ///
    /// This will not have an effect in a slash command's initial response!
    pub fn attachment(&mut self, attachment: serenity::AttachmentType<'att>) -> &mut Self {
        self.attachments.push(attachment);
        self
    }

    /// Toggles whether the message is an ephemeral response (only invoking user can see it).
    ///
    /// This only has an effect in slash commands!
    ///
    /// If this is the initial response and this response
    /// has previously been deferred, the ephemerality is decided by the defer operation. I.e.
    /// if you deferred the response without enabling ephemeral, the initial response will not be
    /// ephemeral.
    pub fn ephemeral(&mut self, ephemeral: bool) -> &mut Self {
        self.ephemeral = ephemeral;
        self
    }

    /// Set the allowed mentions for the message.
    ///
    /// See [`serenity::CreateAllowedMentions`] for more information.
    pub fn allowed_mentions(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateAllowedMentions) -> &mut serenity::CreateAllowedMentions,
    ) -> &mut Self {
        let mut allowed_mentions = serenity::CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the reference message this message is a reply to.
    pub fn reference_message(
        &mut self,
        reference: impl Into<serenity::MessageReference>,
    ) -> &mut Self {
        self.reference_message = Some(reference.into());
        self
    }
}

/// Methods to create a message builder from any type from this [`CreateReply`]. Used by poise
/// internally to actually send a response to Discord
impl<'att> CreateReply<'att> {
    /// Serialize this response builder to a [`serenity::CreateInteractionResponseData`]
    pub fn to_slash_initial_response(self, f: &mut serenity::CreateInteractionResponseData<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reference_message: _, // can't reply to a message in interactions
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
        if let Some(components) = components {
            f.components(|f| {
                f.0 = components.0;
                f
            });
        }
        f.ephemeral(ephemeral);
        f.add_files(attachments);
    }

    /// Serialize this response builder to a [`serenity::CreateInteractionResponseFollowup`]
    pub fn to_slash_followup_response(
        self,
        f: &mut serenity::CreateInteractionResponseFollowup<'att>,
    ) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reference_message: _,
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(components) = components {
            f.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
        f.ephemeral(ephemeral);
        f.add_files(attachments);
    }

    /// Serialize this response builder to a [`serenity::EditInteractionResponse`]
    pub fn to_slash_initial_response_edit(self, f: &mut serenity::EditInteractionResponse) {
        let crate::CreateReply {
            content,
            embeds,
            attachments: _, // no support for attachment edits in serenity yet
            components,
            ephemeral: _, // can't edit ephemerality in retrospect
            allowed_mentions,
            reference_message: _,
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(components) = components {
            f.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
    }

    /// Serialize this response builder to a [`serenity::EditMessage`]
    pub fn to_prefix_edit(self, f: &mut serenity::EditMessage<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reference_message: _, // can't edit reference message afterwards
        } = self;

        // Empty string resets content (happens when user replaces text with embed)
        f.content(content.as_deref().unwrap_or(""));

        f.set_embeds(embeds);

        f.0.insert("attachments", serenity::json::json! { [] }); // reset attachments
        for attachment in attachments {
            f.attachment(attachment);
        }

        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|b| {
                *b = allowed_mentions;
                b
            });
        }

        // When components is None, this will still be run to reset the components.
        f.components(|f| {
            if let Some(components) = components {
                *f = components;
            }
            f
        });
    }

    /// Serialize this response builder to a [`serenity::CreateMessage`]
    pub fn to_prefix(self, m: &mut serenity::CreateMessage<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reference_message,
        } = self;

        if let Some(content) = content {
            m.content(content);
        }
        m.set_embeds(embeds);
        if let Some(allowed_mentions) = allowed_mentions {
            m.allowed_mentions(|m| {
                *m = allowed_mentions;
                m
            });
        }
        if let Some(components) = components {
            m.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(reference_message) = reference_message {
            m.reference_message(reference_message);
        }

        for attachment in attachments {
            m.add_file(attachment);
        }
    }
}

/// Returned from [`send_reply`] to retrieve the sent message object.
///
/// Discord sometimes returns the [`serenity::Message`] object directly, but sometimes you have to
/// request it manually. This enum abstracts over the two cases
pub enum ReplyHandle<'a> {
    /// When sending a normal message or application command followup response, Discord returns the
    /// message object directly
    Known(Box<serenity::Message>),
    /// When sending an initial application command response, you need to request the message object
    /// seperately
    Unknown {
        /// Serenity HTTP instance that can be used to request the interaction response message
        /// object
        http: &'a serenity::Http,
        /// Interaction which contains the necessary data to request the interaction response
        /// message object
        interaction: &'a serenity::ApplicationCommandInteraction,
    },
    /// Reply was attempted to be sent in autocomplete context, resulting in a no-op. Calling
    /// methods on this variant will panic
    Autocomplete,
}

impl ReplyHandle<'_> {
    /// Retrieve the message object of the sent reply.
    ///
    /// Only needs to do an HTTP request in the application command response case
    pub async fn message(self) -> Result<serenity::Message, serenity::Error> {
        match self {
            Self::Known(msg) => Ok(*msg),
            Self::Unknown { http, interaction } => interaction.get_interaction_response(http).await,
            Self::Autocomplete => {
                panic!("reply is a no-op in autocomplete context; can't retrieve message")
            }
        }
    }

    /// Edits the message that this [`ReplyHandle`] points to
    // TODO: return the edited Message object?
    pub async fn edit<'att, U, E>(
        &self,
        ctx: crate::Context<'_, U, E>,
        builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att>,
    ) -> Result<(), serenity::Error> {
        // TODO: deduplicate this block of code
        let mut reply = crate::CreateReply {
            ephemeral: ctx.command().ephemeral,
            allowed_mentions: ctx.framework().options().allowed_mentions.clone(),
            ..Default::default()
        };
        builder(&mut reply);
        if let Some(callback) = ctx.framework().options().reply_callback {
            callback(ctx, &mut reply);
        }

        match self {
            Self::Known(msg) => {
                msg.clone()
                    .edit(ctx.discord(), |b| {
                        reply.to_prefix_edit(b);
                        b
                    })
                    .await?;
            }
            Self::Unknown { http, interaction } => {
                interaction
                    .edit_original_interaction_response(http, |b| {
                        reply.to_slash_initial_response_edit(b);
                        b
                    })
                    .await?;
            }
            Self::Autocomplete => {
                panic!("reply is a no-op in autocomplete context; can't edit message")
            }
        }
        Ok(())
    }
}

/// Send a message in the given context: normal message if prefix command, interaction response
/// if application command.
///
/// If you just want to send a string, use [`say_reply`].
///
/// Note: panics when called in an autocomplete context!
///
/// ```rust,no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ctx: poise::Context<'_, (), ()> = todo!();
/// ctx.send(|f| f
///     .content("Works for slash and prefix commands")
///     .embed(|f| f
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<'att, U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => {
            ReplyHandle::Known(crate::send_prefix_reply(ctx, builder).await?)
        }
        crate::Context::Application(ctx) => crate::send_application_reply(ctx, builder).await?,
    })
}

/// Shorthand of [`send_reply`] for text-only messages
///
/// Note: panics when called in an autocomplete context!
pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into())).await
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::ApplicationCommandInteraction::create_followup_message) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<'att, U, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    let mut data = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        allowed_mentions: ctx.framework.options().allowed_mentions.clone(),
        ..Default::default()
    };
    builder(&mut data);
    _send_application_reply(ctx, data).await
}

/// private version of [`send_application_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_application_reply<'a, 'b, U, E>(
    ctx: crate::ApplicationContext<'b, U, E>,
    mut data: crate::CreateReply<'a>,
) -> Result<crate::ReplyHandle<'b>, serenity::Error> {
    let interaction = match ctx.interaction {
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
            return Ok(crate::ReplyHandle::Autocomplete)
        }
    };

    if let Some(callback) = ctx.framework.options().reply_callback {
        callback(ctx.into(), &mut data);
    }

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    Ok(if has_sent_initial_response {
        crate::ReplyHandle::Known(Box::new(
            interaction
                .create_followup_message(ctx.discord, |f| {
                    data.to_slash_followup_response(f);
                    f
                })
                .await?,
        ))
    } else {
        interaction
            .create_interaction_response(ctx.discord, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|f| {
                        data.to_slash_initial_response(f);
                        f
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        crate::ReplyHandle::Unknown {
            http: &ctx.discord.http,
            interaction,
        }
    })
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'att, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    let mut reply = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        allowed_mentions: ctx.framework.options().allowed_mentions.clone(),
        ..Default::default()
    };
    builder(&mut reply);
    _send_prefix_reply(ctx, reply).await
}

/// private version of [`send_prefix_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_prefix_reply<'a, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    mut reply: crate::CreateReply<'a>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    if let Some(callback) = ctx.framework.options().reply_callback {
        callback(ctx.into(), &mut reply);
    }

    // This must only return None when we _actually_ want to reuse the existing response! There are
    // no checks later
    let lock_edit_tracker = || {
        if ctx.command.reuse_response {
            if let Some(edit_tracker) = &ctx.framework.options().prefix_options.edit_tracker {
                return Some(edit_tracker.write().unwrap());
            }
        }
        None
    };

    let existing_response = lock_edit_tracker()
        .as_mut()
        .and_then(|t| t.find_bot_response(ctx.msg.id))
        .cloned();

    Ok(Box::new(if let Some(mut response) = existing_response {
        response
            .edit(ctx.discord, |f| {
                reply.to_prefix_edit(f);
                f
            })
            .await?;

        // If the entry still exists after the await, update it to the new contents
        if let Some(mut edit_tracker) = lock_edit_tracker() {
            edit_tracker.set_bot_response(ctx.msg, response.clone());
        }

        response
    } else {
        let new_response = ctx
            .msg
            .channel_id
            .send_message(ctx.discord, |m| {
                reply.to_prefix(m);
                m
            })
            .await?;
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.set_bot_response(ctx.msg, new_response.clone());
        }

        new_response
    }))
}
