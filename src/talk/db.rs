use std::ops::Deref;
use actix::prelude::{ Handler, Message, };
use actix_web::{ error, Error, };
use crate::db::{ DbExecutor };
use crate::model;
use crate::model::{ NewIp };
use crate::talk::model::{
    Chat, NewChat, NewSecret, EditChat, HistoryChat, FlagIp, 
};
 
#[derive(Serialize)]
pub struct OneChatOut {
    pub id: i32,
    pub rootnum: i32,
    pub replnum: i32,
    pub posted: String,
    pub whosent: String,
    pub flag: i32,
    pub attached: String,
    pub description: String,
    pub youflagged: bool,
    pub isrep: bool,
}
   
pub struct ChatsOut {
    pub ipid: Option<String>,
    pub res: Vec<OneChatOut>,
}

pub struct ChatsIn {
    pub ipaddr: Option<String>,
    pub ident: Option<String>,
}

impl Message for ChatsIn {
    type Result = Result<ChatsOut, Error>;
}

impl Handler<ChatsIn> for DbExecutor {
    type Result = Result<ChatsOut, Error>;

    fn handle(&mut self, _in: ChatsIn, _: &mut Self::Context) -> Self::Result {
        let mut newipid: Option<String> = None;
        let id: i32 = if let Some(id) = _in.ident {
            id.parse().unwrap_or(0)
        } else {
            match _in.ipaddr {
                Some(addr) => {
                    let di = model::upsertip(NewIp{ipaddr: addr}, self.get_conn()?.deref()).unwrap();
                    println!("FROM UPSERT IP {:?}", di);
                    let tt = di.id.to_owned();
                    newipid = Some(tt.to_string());
                    tt
                },
                None => 0,
            }
        };
        let mut vec = Vec::new();
        let cs = Chat::readchats(self.get_conn()?.deref(), 0)
           .map_err(|_| error::ErrorInternalServerError("Error reading chats"));
        match cs {
            Ok(chats) => {
                match chats.len() {
                    0 => {
                    },
                    _ => {
                        let mut prevroot: i32 = 0;
                        for t in chats {
                            let att: String = match &t.attached {
                                Some(link) => link.to_string(),
                                None => "none".to_string(),
                            };
                            let yf: bool = if t.flag > 0 {
                                let uflag = Chat::you_flagged_this_chat(self.get_conn()?.deref(), &t, &id);
                                match uflag {
                                    Ok(vec) => {
                                        match vec.len() {
                                            0 => false,
                                            _ => true,
                                        }
                                    },
                                    Err(_) => false,
                                }
                            } else {
                                false
                            };
                            let ir: bool = if t.rootnum == prevroot {
                                true
                            } else {
                                prevroot = t.rootnum;
                                false
                            };
                            let out = OneChatOut {
                                id: t.id.to_owned(),
                                rootnum: t.rootnum.to_owned(),
                                replnum: t.replnum.to_owned(),
                                posted: t.timeposted.to_owned().to_string()[..10].to_string(),
                                whosent: t.whosent.to_owned().to_string(),
                                flag: t.flag.to_owned(),
                                attached: att,
                                description: t.description.to_owned().to_string(),
                                youflagged: yf,
                                isrep: ir,
                            };
                            vec.push(out);
                        }
                    },
                }
            },
            Err(_) => (),
        }
        Ok(ChatsOut{ipid: newipid, res: vec})
    }
}

pub struct ReadChats {
    pub offset: i64,
    pub ipid: i32,
}

impl Message for ReadChats {
    type Result = Result<Vec<OneChatOut>, Error>;
}

impl Handler<ReadChats> for DbExecutor {
    type Result = Result<Vec<OneChatOut>, Error>;

    fn handle(&mut self, at: ReadChats, _: &mut Self::Context) -> Self::Result {
        let res = Chat::readchats(self.get_conn()?.deref(), at.offset)
            .map_err(|_| error::ErrorInternalServerError("Error reading chats"));
        match res {
            Ok(chats) => {
                let mut vec = Vec::new();
                match chats.len() {
                    0 => {
                    },
                    _ => {
                        let mut prevroot: i32 = 0;
                        for t in chats {
                            let att: String = match &t.attached {
                                Some(link) => link.to_string(),
                                None => "none".to_string(),
                            };
                            let yf: bool = if t.flag > 0 {
                                let uflag = Chat::you_flagged_this_chat(self.get_conn()?.deref(), &t, &at.ipid);
                                match uflag {
                                    Ok(vec) => {
                                        match vec.len() {
                                            0 => false,
                                            _ => true,
                                        }
                                    },
                                    Err(_) => false,
                                }
                            } else {
                                false
                            };
                            let ir: bool = if t.rootnum == prevroot {
                                true
                            } else {
                                prevroot = t.rootnum;
                                false
                            };
                            let out = OneChatOut {
                                id: t.id.to_owned(),
                                rootnum: t.rootnum.to_owned(),
                                replnum: t.replnum.to_owned(),
                                posted: t.timeposted.to_owned().to_string()[..10].to_string(),
                                whosent: t.whosent.to_owned().to_string(),
                                flag: t.flag.to_owned(),
                                attached: att,
                                description: t.description.to_owned().to_string(),
                                youflagged: yf,
                                isrep: ir,
                            };
                            vec.push(out);
                        }
                    },
                }
                Ok(vec)
            },
            Err(e) => Err(e),
       }
    }
}

pub struct CreateChat {
    pub inheritedid: Option<i32>,
    pub secret: String,
    pub whosent: String,
    pub ip: String,
    pub linky: Option<String>,
    pub description: String,
}

impl Message for CreateChat {
    type Result = Result<Chat, Error>;
}

impl Handler<CreateChat> for DbExecutor {
    type Result = Result<Chat, Error>;

    fn handle(&mut self, c: CreateChat, _: &mut Self::Context) -> Self::Result {
        let uip = model::upsertip( NewIp{ ipaddr: c.ip }, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        let ip_id = match uip {
            Ok(done) => {
                done.id
            },
            Err(_) => 0,
        };
        let new_chat = NewChat {
            ipid: ip_id,
            whosent: c.whosent,
            attached: c.linky,
            description: c.description,
        };
        let ins = Chat::inserttask(new_chat, self.get_conn()?.deref())
           .map_err(|_| error::ErrorInternalServerError("Error insert"));
        match ins {
            Ok(chat) => {
                let _ = match c.inheritedid.as_ref() {
                    Some(parentid) => {
                        let replnum = Chat::get_max_replnum(*parentid, self.get_conn()?.deref())
                            .map_err(|_| error::ErrorInternalServerError("Error get_max_replynum"));
                        match replnum {
                            Ok(num) => {
                                let new: i32 = num + 1;
                                let _ = Chat::set_as_repl(chat.id, *parentid, new, self.get_conn()?.deref())
                                    .map(|_| ())
                                    .map_err(|_| error::ErrorInternalServerError("Error set_as_repl"));
                            },
                            Err(_) => (),
                        };
                    },
                    None => {
                        let _ = Chat::set_as_root(chat.id, self.get_conn()?.deref())
                            .map(|_| ())
                            .map_err(|_| error::ErrorInternalServerError("Error set_as_root"));
                    }
                };
                let new_secret = NewSecret {
                    secret: c.secret,
                    chatid: chat.id,
                };
                let _ = Chat::insertsecret(new_secret, self.get_conn()?.deref())
                    .map(|_| ())
                    .map_err(|_| error::ErrorInternalServerError("Error inserting secret"));
                Chat::get_chat(chat.id, self.get_conn()?.deref())
                    .map_err(|_| error::ErrorInternalServerError("Error set_as_root"))
            },
            Err(e) => Err(e),
        }
    }
}

pub struct ToggleChat {
    pub id: i32,
    pub pw: String,
}

impl Message for ToggleChat {
    type Result = Result<i32, Error>;
}

impl Handler<ToggleChat> for DbExecutor {
    type Result = Result<i32, Error>;

    fn handle(&mut self, c: ToggleChat, _: &mut Self::Context) -> Self::Result {
        let pw = Chat::get_secret(c.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        match pw {
            Ok(secret) => {
                if secret == c.pw {
                    Ok(c.id)
                } else {
                    // wrong password
                    Ok(0)
                }
            },
            Err(e) => Err(e),
        }
    }
}

pub struct DeleteChat {
    pub id: i32,
    pub pw: String,
}

impl Message for DeleteChat {
    type Result = Result<usize, Error>;
}

impl Handler<DeleteChat> for DbExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, chat: DeleteChat, _: &mut Self::Context) -> Self::Result {
        let pw = Chat::get_secret(chat.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        match pw {
            Ok(secret) => {
                if secret == chat.pw {
                    let t = Chat::get_chat(chat.id, self.get_conn()?.deref())
                        .map_err(|_| error::ErrorInternalServerError("Error getting task"));
                    match t {
                        Ok(sk) => {
                            let _ = Chat::inserthistory(HistoryChat{
                                    chatid: sk.id,
                                    ipid: sk.ipid,
                                    whathappened: "deleted".to_string(),
                                    rootnum: sk.rootnum,
                                    replnum: sk.replnum,
                                    timeposted: sk.timeposted,
                                    whosent: sk.whosent,
                                    flag: sk.flag,
                                    attached: sk.attached,
                                    description: sk.description,
                                }, self.get_conn()?.deref())
                                .map_err(|_| error::ErrorInternalServerError("Error inserting historytask"));
                        }, 
                        Err(_) => (),
                    };
                    Chat::delete_with_id(chat.id, self.get_conn()?.deref())
                        .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
                } else {
                    // wrong password
                    Ok(0)
                }
            },
            Err(e) => Err(e),
        }
    }
}

impl Message for EditChat {
    type Result = Result<usize, Error>;
}

impl Handler<EditChat> for DbExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, chat: EditChat, _: &mut Self::Context) -> Self::Result {

        let t = Chat::get_chat(chat.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error getting task"));
        match t {
            Ok(sk) => {
                let _ = Chat::inserthistory(HistoryChat{
                    chatid: sk.id,
                    ipid: sk.ipid,
                    whathappened: "edited".to_string(),
                    rootnum: sk.rootnum,
                    replnum: sk.replnum,
                    timeposted: sk.timeposted,
                    whosent: sk.whosent,
                    flag: sk.flag,
                    attached: sk.attached,
                    description: sk.description,
                }, self.get_conn()?.deref())
                .map_err(|_| error::ErrorInternalServerError("Error inserting historytask"));
             }, 
             Err(_) => (),
        };
        Chat::re_write_desc(&chat, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
    }
}

pub struct FlagChat {
    pub ip: String,
    pub chatid: i32,
    pub dir: String,
}

impl Message for FlagChat {
    type Result = Result<i32, Error>;
}

impl Handler<FlagChat> for DbExecutor {
    type Result = Result<i32, Error>;

    fn handle(&mut self, ch: FlagChat, _: &mut Self::Context) -> Self::Result {
        let uip = model::upsertip( NewIp{ ipaddr: ch.ip }, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        let ip_id = match uip {
            Ok(done) => {
                done.id
            },
            Err(_) => 0,
        };
        let f;
        if ch.dir == "-" {
            f = Chat::sub_flagnum(ch.chatid, self.get_conn()?.deref())
                .map_err(|_| error::ErrorInternalServerError("Error getting task"));
            match f {
                Ok(_) => {
                    let _ = Chat::rm_who_flagged(FlagIp{
                        ipid: ip_id,
                        chatid: ch.chatid,
                    }, self.get_conn()?.deref())
                    .map_err(|_| error::ErrorInternalServerError("Error who flagged"));
                }, 
                Err(_) => (),
            }   
        } else {
            f = Chat::add_flagnum(ch.chatid, self.get_conn()?.deref())
                .map_err(|_| error::ErrorInternalServerError("Error getting task"));
            match f {
                Ok(_) => {
                    let _ = Chat::who_flagged(FlagIp{
                        ipid: ip_id,
                        chatid: ch.chatid,
                    }, self.get_conn()?.deref())
                    .map_err(|_| error::ErrorInternalServerError("Error who flagged"));
                },
                Err(_) => (),
            }
        }
        Chat::get_flag(ch.chatid, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error flagging")) 
    }
}
