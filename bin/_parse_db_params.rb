#!/usr/bin/env ruby
require 'uri'

uri = URI(ARGV[0])

result = {}
unless uri.userinfo.nil?
  username, password = uri.userinfo.split(':') 
  result["DB_USERNAME"] = username
  result["DB_PASSWORD"] = password
end

result["DB_PORT"] = uri.port unless uri.port.nil?

database = uri.path.sub(/\//,'')

raise RuntimeError, "need at lease a database and host" if uri.host.nil? || database.nil?

result["DB_HOST"] = uri.host
result["DB_NAME"] = database

for key in result.keys
  puts "#{key}=#{result[key]}"
end
