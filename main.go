package main

import "github.com/firefly-zero/firefly-go/firefly"

var (
	peers       firefly.Peers
	font        firefly.Font
	frame       uint32
	me          firefly.Peer
	oldBtns     firefly.Buttons
	stopped     bool
	dialogRight bool
)

const (
	FontHeight = 10
	FontWidth  = 6
	X          = 120 - 3*16
	Y          = 50
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
	font = firefly.LoadFile("font", nil).Font()
}

func update() {
	frame += 1
	me = firefly.GetMe()
	newPeers := firefly.GetPeers()
	if newPeers != peers {
		peers = newPeers
	}

	handlePad()
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
	if stopped && !newBtns.S && oldBtns.S {
		if dialogRight {
			setConnStatus(connFinished)
		} else {
			setConnStatus(connCancelled)
		}
		return
	}

}

func handlePad() {
	newPad, _ := firefly.ReadPad(me)
	newDPad := newPad.DPad()
	if newDPad.Left {
		dialogRight = false
	}
	if newDPad.Right {
		dialogRight = true
	}
}

func render() {
	firefly.ClearScreen(firefly.ColorWhite)
	if !stopped {
		drawConnecting()
	}
	drawPeers()
	drawButtons()
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

func drawButtons() {
	margin := 46
	corner := firefly.Size{W: 4, H: 4}
	boxStyle := firefly.Style{
		StrokeColor: firefly.ColorDarkBlue,
		StrokeWidth: 1,
	}
	boxWidth := firefly.Width - margin*2
	btnWidth := FontWidth * 7
	y := 120

	if !stopped {
		x := (firefly.Width - btnWidth) / 2
		point := firefly.Point{X: x + 3, Y: y + 7}
		firefly.DrawText(" stop", font, point, firefly.ColorDarkBlue)
		if !dialogRight {
			firefly.DrawRoundedRect(
				firefly.Point{X: x, Y: y},
				firefly.Size{W: btnWidth, H: 12},
				corner,
				boxStyle,
			)
		}
	}

	if stopped {
		x := margin + boxWidth/2 - (btnWidth + btnWidth/2)
		point := firefly.Point{X: x + 3, Y: y + 7}
		firefly.DrawText("cancel", font, point, firefly.ColorDarkBlue)
		if !dialogRight {
			firefly.DrawRoundedRect(
				firefly.Point{X: x, Y: y},
				firefly.Size{W: btnWidth, H: 12},
				corner,
				boxStyle,
			)
		}
	}

	if stopped {
		x := margin + boxWidth/2 + btnWidth/2
		point := firefly.Point{X: x + 3, Y: y + 7}
		firefly.DrawText("  ok", font, point, firefly.ColorDarkBlue)
		if dialogRight {
			firefly.DrawRoundedRect(
				firefly.Point{X: x, Y: y},
				firefly.Size{W: btnWidth, H: 12},
				corner,
				boxStyle,
			)
		}
	}

}
