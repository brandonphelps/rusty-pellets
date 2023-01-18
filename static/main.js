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

class ControllerApp extends React.Component {
    constructor(props) {
	super(props);
	this.state = {
	    angle_one: 0,
	    angle_two: 0
	};
    }

    componentDidMount() {
	var ws = new WebSocket('ws://' + window.location.host + '/ws');
	var that = this;
	ws.onmessage = function(event) {

	    // todo: maybe move this to some sort of processing function?
	    const response = JSON.parse(event.data);
	    if (response.t == "ServoState") {
		const angle_one = response.c[0].angle;
		const angle_two = response.c[1].angle;
		that.setState(
		    {
			angle_one: angle_one,
			angle_two: angle_two
		    }
		);
	    }
	}
    }

    render() {
	// needs a disconnect button.
	return (
	    <div className="controller-app">
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
	<Game />
	<ControllerApp />
    </div>);


import foo from './foo.js';

export default function() {
    console.log(foo);
}
