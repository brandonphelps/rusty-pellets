'use strict';

import React from 'react';
import ReactDOM from 'react-dom/client';
import axios from 'axios';

// Bootstrap CSS
import "bootstrap/dist/css/bootstrap.min.css";
// Bootstrap Bundle JS
import "bootstrap/dist/js/bootstrap.bundle.min";

class Square extends React.Component {
    render() {
	return(
	    <button className="square"
		    onClick={ () => this.props.onClick()}
	    >
		{ this.props.value } 
	    </button>
	);
    }
}


function calculateWinner(squares) {
    const lines = [
	[0, 1, 2],
	[3, 4, 5],
	[6, 7, 8],
	[0, 3, 6],
	[1, 4, 7],
	[2, 5, 8],
	[0, 4, 8],
	[2, 4, 6],
    ];

    // console.log("calculate winner");

    for (let i =0; i < lines.length; i++) {
	const [a, b, c] = lines[i];
	if (squares[a] && squares[a] === squares[b] && squares[a] === squares[c]) {
	    return squares[a];
	}
    }

    return null;
}

class Board extends React.Component {
    renderSquare(i) {
	return (
	    <Square
		value={this.props.squares[i]}
		onClick={() => this.props.onClick(i)}
	    />
	);
    }


    render() {
	return (
	    <div>
		<div className="board-row">
		    {this.renderSquare(0)}
		    {this.renderSquare(1)}
		    {this.renderSquare(2)}
		</div>
		<div className="board-row">
		    {this.renderSquare(3)}
		    {this.renderSquare(4)}
		    {this.renderSquare(5)}
		</div>
		<div className="board-row">
		    {this.renderSquare(6)}
		    {this.renderSquare(7)}
		    {this.renderSquare(8)}
		</div>
	    </div>
	);
    }
}

class Game extends React.Component {
    constructor(props) { 
	super(props);
	this.state = {
	    history: [{
		squares: Array(9).fill(null),
	    }],
	    stepNumber: 0,
	    xIsNext: true,
	};
    }

    jumpTo(step) {
	this.setState({
	    stepNumber: step,
	    xIsNext: (step % 2) === 0,
	});
    }

    handleClick(i) {
	console.log("Handle click");
	const history = this.state.history.slice(0, this.state.stepNumber + 1);
	const current = history[this.state.stepNumber];
	const squares = current.squares.slice();
	if (calculateWinner(squares) || squares[i]) {
	    return;
	}

	squares[i] = this.state.xIsNext ? 'X' : 'O';
	this.setState(
	    {
		history: history.concat([{
		    squares: squares,
		}]),
		stepNumber: history.length,
		xIsNext: !this.state.xIsNext,
	    }
	);
    }

    render() {
	const history = this.state.history;
	const current = history[this.state.stepNumber];
	const winner = calculateWinner(current.squares);

	const moves = history.map((step, move) => {
	    const desc = move ?
		  'Go to move #' + move : 
		  'Go to game start';
	    return (
		<li key={move}>
		    <button onClick={() => this.jumpTo(move)}>{desc}</button>
		</li>
	    );
	});

	let status;
	if (winner) {
	    status = 'Winner: ' + winner;
	} else {
	    status = 'Next player: ' + (this.state.xIsNext ? 'X' : 'O');
	}

	return (
	    <div className="game">
		<div className="game-board">
		    <Board
			squares={current.squares}
			onClick={(i) => this.handleClick(i)}
		    />
		</div>
		<div className="game-info">
		    <div>{status}</div>
		    <ol>{moves}</ol>
		</div>
	    </div>
	);
    }
}



// This is a gauge indicating the angle of a single servo
class ServoGauge extends React.Component {
    constructor(props) {
	super(props);
	// todo: constructor? should contain
	// some reference to a servo id that is
	// unique to the servo.
    }
    
    render() {
	const label_value = 'Servo ' + this.props.servo_id;
	return (
	    <div className="servo">
		<div>{label_value}</div>
		<textarea readOnly value={this.props.angle}></textarea>
	    </div>
	);
    }
}

class ConnectedStatusGuage extends React.Component {
    render() {
	return (
	    <div className="controller-status">
		<div>Connected</div>
		<div>Disconnected</div>
	    </div>
	);
    }
}

class ControllerButton extends React.Component {
    render() {
	return (
	    <button onClick={() => this.props.onClick()} className="controller-status-button btn btn-primary">
		Connect
	    </button>
	);
    }
}

class ControllerState {
    constructor() {
	this.state = {
	    input_state: {
		'up': false,
		'down': false,
		'left': false,
		'right': false,
	    },
	};
	// todo: ensure this gets cleaned up.
	window.addEventListener('keydown', (e) => { this.key_down_update(e) });
	window.addEventListener('keyup', (e) => { this.key_up_update(e) });
    }

    key_down_update(event) {
	console.log("Key down event: " + JSON.stringify(this.state));
	if (event.key == 'w') { 
	    this.state.input_state.up = true;
	} 
	else if (event.key == 'd') { 
	    this.state.input_state.down = true;
	} 
	else if (event.key == 'a') { 
	    this.state.input_state.left = true;
	} 
	else if (event.key == 's') { 
	    this.state.input_state.right = true;
	} else { 
 	    // no valid key reject 
 	    // todo: display message to  user? 
 	    return; 
	}
    }

    key_up_update(event) {
	console.log("Key up event: " + JSON.stringify(this.state));
	if (event.key == 'w') { 
	    this.state.input_state.up = false;
	} 
	else if (event.key == 'd') { 
	    this.state.input_state.down = false;
	} 
	else if (event.key == 'a') { 
	    this.state.input_state.left = false;
	} 
	else if (event.key == 's') { 
	    this.state.input_state.right = false;
	} else { 
 	    // no valid key reject 
 	    // todo: display message to  user? 
 	    return; 
	}
    }
}

class ControllerApp extends React.Component {
    constructor(props) {
	super(props);
	this.state = {
	    angle_one: 0,
	    angle_two: 0,
	    connected: false,
	    button_text: 'Connect',
	};

	this.controller_state = new ControllerState();
    }

    build_websocket() {
	// prob not needed. 
	var that = this;
	var web_sock = new WebSocket('ws://' + window.location.host + '/ws');

	web_sock.onopen = () => {
	    console.log("Connected websocket");
	    this.setState({web_sock: web_sock});

	    // start timer on connect, this will continuously send
	    // the recorded key state. 
	    this.timer = setInterval(() => {
		console.log("Current state " + JSON.stringify(this.controller_state));

		let input_state = {'t': 'Servo', 'c': this.controller_state.state.input_state};
		try {

		    this.state.web_sock.send(JSON.stringify(input_state));
		} catch (error) {
		    console.log("Failed to send");
		    console.log(error);
		}
	    }, 200);
	};
	
	web_sock.onmessage = function(event) {
	    // todo: maybe move this to some sort of processing function?
	    const response = JSON.parse(event.data);
	    if (response.t == "ServoState") {
		const angle_one = response.c[0].angle;
		const angle_two = response.c[1].angle;
		that.setState(
		    {
			angle_one: angle_one,
			angle_two: angle_two,
		    }
		);
	    }
	};

	web_sock.onclose = function(event) {
	    console.log('The connection has been closed successfully');
	    // todo: do the same sort of disconnect logic here.
	    // however since it closed we don't want to do web_sock stuff.
	};
    }

    connect() {
	// websocket doesn't provide a re open
	// how does this affect the button? does the button still bind to the old websocket?
	console.log("Connecting to server ws");
	this.build_websocket();
	// todo should likely check return values from connecting to the server.
	this.setState( {
	    connected: true,
	    button_text: 'Disconnect',
	});
    }

    disconnect() {
	console.log("Disconnecting to server ws");
	clearInterval(this.timer);
	this.state.web_sock.send(JSON.stringify({"t": "Disconnect"}));
	this.setState({
	    connected: false,
	    button_text: 'Connect',
	});
    }
    
    componentDidMount() {
	
    }

    render() {
	// needs a disconnect button.
	var button_callback;
	if (!this.state.connected) {
	    button_callback = () => { this.connect() };
	} else {
	    button_callback = () => { this.disconnect() };
	}

	return (
	    <div className="controller-app">
		<ConnectedStatusGuage
		    
		/>
		<textarea></textarea>

		<button onClick={ button_callback } className="controller-status-button btn btn-primary">
		    { this.state.button_text }
		</button>

		<ServoGauge
		    angle={this.state.angle_one}
		    servo_id={1}
		/>
		<ServoGauge
		    angle={this.state.angle_two}
		    servo_id={2}
		/>
	    </div>
	);
    }
}

const root = ReactDOM.createRoot(document.querySelector("#root"));
root.render(
    <div>
	<ControllerApp />
	<br />
	<br />
	<Game />
    </div>);


