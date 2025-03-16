package main

import "github.com/firefly-zero/firefly-go/firefly"

var (
	peers   firefly.Peers
	font    firefly.Font
	frame   uint32
	me      firefly.Peer
	oldBtns firefly.Buttons
	stopped bool
)

const (
	FontHeight = 10
	FontWidth  = 6
	X          = 120 - 3*13
	Y          = 71
)

const (
	connStopped   = 1
	connCancelled = 2
	connFinished  = 3
)

func init() {
	firefly.Boot = boot
	firefly.Update = update
	firefly.Render = render
}

func boot() {
	buf := [655]byte{}
	font = firefly.LoadFile("font", buf[:]).Font()
}

func update() {
	frame += 1
	me = firefly.GetMe()
	newPeers := firefly.GetPeers()
	if newPeers != peers {
		peers = newPeers
	}

	newBtns := firefly.ReadButtons(me)
	handleButtons(newBtns)
	oldBtns = newBtns
}

func handleButtons(newBtns firefly.Buttons) {
	// If a button is pressed, just track it and return.
	// All actions the module does happen on button release, not press.
	if newBtns.AnyPressed() {
		return
	}

	// Connecting is not stopped, a button was pressed
	// but is released now. Stop connecting.
	if !stopped && oldBtns.AnyPressed() {
		stopped = true
		setConnStatus(connStopped)
		return
	}

	// Connecting is stopped. The user either confirms that all
	// connected devices are good or cancels.
	if stopped {
		// confirm
		if !newBtns.S && oldBtns.S {
			setConnStatus(connFinished)
			return
		}
		// cancel
		if !newBtns.E && oldBtns.E {
			setConnStatus(connCancelled)
			return
		}
	}

}

func render() {
	firefly.ClearScreen(firefly.ColorWhite)
	if !stopped {
		drawConnecting()
	}
	drawPeers()
}

func drawConnecting() {
	point := firefly.Point{X: X, Y: Y - FontHeight}
	text := "Connecting..."
	firefly.DrawText(text, font, point, firefly.ColorLightGray)

	step := int(frame) / 5
	var shift int
	length := len(text) + 1
	if step%(length*2) >= length {
		shift = step % length
		text = text[step%length:]
	} else {
		shift = 0
		text = text[:step%length]
	}
	point = firefly.Point{X: X + shift*FontWidth, Y: Y - FontHeight}
	firefly.DrawText(text, font, point, firefly.ColorBlack)
}

func drawPeers() {
	i := 0
	for peer := range peers.Iter() {
		name := firefly.GetName(peer)
		if name == "" {
			name = "<empty>"
		}
		point := firefly.Point{X: X, Y: Y + FontHeight*(i+1)}
		if peer == me {
			firefly.DrawText("you:", font, point, firefly.ColorBlue)
			point.X += FontWidth * 5
		}
		firefly.DrawText(name, font, point, firefly.ColorBlack)
		i++
	}
}
