var React = require('react');

var HelloWorld = React.createClass({
    render: function() {
        return <div>Hello {this.props.name}</div>;
    }
});

module.exports = HelloWorld;
