DROP TABLE IF EXISTS analytics;
CREATE TABLE analytics (
  id         serial primary key,
  name       varchar(40),
  data       jsonb,
  CONSTRAINT production UNIQUE(name)
);
