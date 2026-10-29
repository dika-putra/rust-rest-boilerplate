-- ids are generated in Rust (uuid v4, stored as 16-byte blobs) and timestamps are bound
-- from Rust, so sqlx can decode them back into OffsetDateTime
create table if not exists users
(
    id         blob primary key not null,
    name       text not null default '',
    email      text not null default '' collate nocase,
    password   text not null default '',
    created_at text not null,
    updated_at text not null
);

create unique index if not exists users_email_idx on users (email);

create table if not exists categories
(
    id         blob primary key not null,
    name       text not null default '',
    cat_type   text not null default 'NonEssential' check (cat_type in ('Essential', 'NonEssential')),
    user_id    blob not null references users (id) on delete cascade,
    created_at text not null,
    updated_at text not null
);

create table if not exists sessions
(
    id         blob primary key not null,
    exp        text not null,
    user_id    blob not null references users (id) on delete cascade,
    user_agent text not null default ''
);
