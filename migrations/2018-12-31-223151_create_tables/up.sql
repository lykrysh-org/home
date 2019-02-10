create table ipaddrs (
  id SERIAL PRIMARY KEY,
  ipaddr VARCHAR UNIQUE NOT NULL,
  timefirst TIMESTAMPTZ NOT NULL default current_timestamp,
  timelast TIMESTAMPTZ NOT NULL default current_timestamp
);

CREATE TABLE chats (
  id SERIAL PRIMARY KEY, 
  ipid INT NOT NULL,
  rootnum INT NOT NULL DEFAULT 1,
  replnum INT NOT NULL DEFAULT 1,
  timeposted TIMESTAMPTZ NOT NULL default current_timestamp,
  whosent VARCHAR NOT NULL,
  flag INT NOT NULL DEFAULT 0,
  attached VARCHAR DEFAULT 'none',
  description VARCHAR NOT NULL,
  foreign key (ipid) references ipaddrs(id) on delete cascade
);

CREATE TABLE csecrets (
  id SERIAL PRIMARY KEY,
  secret VARCHAR NOT NULL,
  chatid INT NOT NULL,
  foreign key (chatid) references chats(id) on delete cascade
);

CREATE TABLE cflags (
  id SERIAL PRIMARY KEY,
  ipid INT NOT NULL,
  chatid INT NOT NULL,
  timeflagged TIMESTAMPTZ NOT NULL default current_timestamp,
  foreign key (ipid) references ipaddrs(id) on delete cascade,
  foreign key (chatid) references chats(id) on delete cascade
);

CREATE TABLE historychats (
  id SERIAL PRIMARY KEY,
  chatid INT NOT NULL,
  ipid INT NOT NULL,
  whathappened VARCHAR NOT NULL,
  timehappened TIMESTAMPTZ NOT NULL default current_timestamp,
  rootnum INT NOT NULL,
  replnum INT NOT NULL,
  timeposted TIMESTAMPTZ NOT NULL,
  whosent VARCHAR NOT NULL,
  flag INT NOT NULL,
  attached VARCHAR DEFAULT 'none',
  description VARCHAR NOT NULL
);

CREATE TABLE shows (
  id SERIAL PRIMARY KEY, 
  imgnum INT NOT NULL,
  title VARCHAR NOT NULL,
  year INT NOT NULL,
  intro VARCHAR NOT NULL,
  limitdate date default null,
  popular INT NOT NULL default 0,
  mature boolean default false not null,
  movin boolean default false not null,
  still boolean default false not null,
  graph boolean default false not null,
  anime boolean default false not null,
  illeg boolean default false not null,
  cat1 boolean default false not null,
  cat2 boolean default false not null,
  cat3 boolean default false not null,
  cat4 boolean default false not null
);

create table smakers (
  id serial primary key,
  name text not null,
  showid int not null,
  foreign key (showid) references shows(id) on delete cascade
);

create table sclicks (
  id serial primary key,
  showid int not null,
  ipid int not null,
  timeclicked TIMESTAMPTZ NOT NULL default current_timestamp,
  timeleft TIMESTAMPTZ NOT NULL default current_timestamp,
  foreign key (showid) references shows(id) on delete cascade,
  foreign key (ipid) references ipaddrs(id) on delete cascade
);

create table spages (
  id serial primary key,
  showid int not null,
  mediahost VARCHAR default null,
  mediaid VARCHAR default null,
  reference VARCHAR default null,
  ends int default null,
  foreign key (showid) references shows(id) on delete cascade
);