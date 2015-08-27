DROP TABLE IF EXISTS analytics;
CREATE TABLE analytics (
  id         serial primary key,
  name       varchar(40),
  event_data       jsonb,
  date_created timestamp default current_timestamp
);
