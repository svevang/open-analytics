DROP TABLE IF EXISTS analytics;
CREATE TABLE analytics (
  id         serial primary key,
  name       varchar(40),
  event_data       jsonb,
  date_created timestamp with time zone default current_timestamp
);
