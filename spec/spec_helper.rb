require 'pry'
require 'json'
require "net/http"
require "uri"

API_URL = "http://#{`boot2docker ip`.strip()}:7676"
TEST_DB_CONTAINER = 'open-analytics-db-test'
TEST_OA_CONTAINER = 'open-analytics-test'

module OAExtensions

  def get(path)
    @response = Net::HTTP.get_response(URI.join(API_URL, path))
    @response.define_singleton_method :json, lambda { JSON.parse(body)}
    @response
  end
end

def clean_up_container(container)
  if system("docker ps -a | grep #{container} > /dev/null")
    `docker stop -t 0 #{container}`
    `docker rm #{container}`
  end
end


RSpec.configure do |config|

  config.include OAExtensions
  config.before :suite do

    `docker build -t svevang/open-analytics-test #{File.join(File.dirname(__FILE__),'..')}`

    clean_up_container(TEST_DB_CONTAINER)
    clean_up_container(TEST_OA_CONTAINER)

    `docker run --name #{TEST_DB_CONTAINER} -d svevang/open-analytics-db`

    system("#{"docker run --name #{TEST_OA_CONTAINER} --link #{TEST_DB_CONTAINER}:db " +
               "-e DB_URL=postgres://postgres@db/open_analytics -d -p 7676:3000 " +
               "svevang/open-analytics-test"} > /dev/null")

    host_up = false
    until host_up do
      begin
        host_up = Net::HTTP.get_response(URI(API_URL)).code == "200"
      rescue
      end
    end
  end
  config.after :suite do
    clean_up_container(TEST_DB_CONTAINER)
    clean_up_container(TEST_OA_CONTAINER)
  end
  config.after do
    @response = nil
  end
end
