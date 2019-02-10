table! {
    cflags (id) {
        id -> Int4,
        ipid -> Int4,
        chatid -> Int4,
        timeflagged -> Timestamptz,
    }
}

table! {
    chats (id) {
        id -> Int4,
        ipid -> Int4,
        rootnum -> Int4,
        replnum -> Int4,
        timeposted -> Timestamptz,
        whosent -> Varchar,
        flag -> Int4,
        attached -> Nullable<Varchar>,
        description -> Varchar,
    }
}

table! {
    csecrets (id) {
        id -> Int4,
        secret -> Varchar,
        chatid -> Int4,
    }
}

table! {
    historychats (id) {
        id -> Int4,
        chatid -> Int4,
        ipid -> Int4,
        whathappened -> Varchar,
        timehappened -> Timestamptz,
        rootnum -> Int4,
        replnum -> Int4,
        timeposted -> Timestamptz,
        whosent -> Varchar,
        flag -> Int4,
        attached -> Nullable<Varchar>,
        description -> Varchar,
    }
}

table! {
    ipaddrs (id) {
        id -> Int4,
        ipaddr -> Varchar,
        timefirst -> Timestamptz,
        timelast -> Timestamptz,
    }
}

table! {
    sclicks (id) {
        id -> Int4,
        showid -> Int4,
        ipid -> Int4,
        timeclicked -> Timestamptz,
        timeleft -> Timestamptz,
    }
}

table! {
    shows (id) {
        id -> Int4,
        imgnum -> Int4,
        title -> Varchar,
        year -> Int4,
        intro -> Varchar,
        limitdate -> Nullable<Date>,
        popular -> Int4,
        mature -> Bool,
        movin -> Bool,
        still -> Bool,
        graph -> Bool,
        anime -> Bool,
        illeg -> Bool,
        cat1 -> Bool,
        cat2 -> Bool,
        cat3 -> Bool,
        cat4 -> Bool,
    }
}

table! {
    smakers (id) {
        id -> Int4,
        name -> Text,
        showid -> Int4,
    }
}

table! {
    spages (id) {
        id -> Int4,
        showid -> Int4,
        mediahost -> Nullable<Varchar>,
        mediaid -> Nullable<Varchar>,
        reference -> Nullable<Varchar>,
        ends -> Nullable<Int4>,
    }
}

joinable!(cflags -> chats (chatid));
joinable!(cflags -> ipaddrs (ipid));
joinable!(chats -> ipaddrs (ipid));
joinable!(csecrets -> chats (chatid));
joinable!(sclicks -> ipaddrs (ipid));
joinable!(sclicks -> shows (showid));
joinable!(smakers -> shows (showid));
joinable!(spages -> shows (showid));

allow_tables_to_appear_in_same_query!(
    cflags,
    chats,
    csecrets,
    historychats,
    ipaddrs,
    sclicks,
    shows,
    smakers,
    spages,
);
