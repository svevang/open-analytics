require 'pry'
require 'json'
require "net/http"
require "uri"

API_URL = 'http://localhost:3000'

module OAExtensions

  def get(path)
    @response = Net::HTTP.get_response(URI.join(API_URL, path))
    @response.define_singleton_method :json, lambda { JSON.parse(body)}
    @response
  end
end

RSpec.configure do |config|

  config.include OAExtensions
  config.before :suite do
    #DatabaseCleaner.clean_with(:truncation)
  end
  config.after do
    @response = nil
  end
end
