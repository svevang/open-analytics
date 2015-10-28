require 'spec_helper'
 
describe "events analytics api" do
    context "retrieves a list of events from an event namespace" do
      subject { @response.json }
      before { get '/api/v1/events/foo' }
      it { should == [] }
    end
    context "looking up a non-existent path" do
      subject { @response.code }
      before { get '/api/v1/blah/foo' }
      it { should == "404" }
    end
    context "looking up a static file path" do
      before { get '/' }
      it { expect(@response.code).to eq("200") }
      it { expect(@response.body.strip()).to eq("Hello World") }
    end
end
