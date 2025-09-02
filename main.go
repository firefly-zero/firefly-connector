package main

import "github.com/firefly-zero/firefly-go/firefly"

var (
	peers       firefly.Peers
	font        firefly.Font
	splash      firefly.Image
	frame       uint32
	exited      bool
	me          firefly.Peer
	oldBtns     firefly.Buttons
	stopped     bool
	dialogRight bool
)

const (
	FontHeight = 10
	FontWidth  = 6
	btnWidth   = FontWidth * 7
	X          = (firefly.Width - FontWidth*16) / 2
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
	splash = firefly.LoadFile("_splash", nil).Image()
}

func update() {
	if exited {
		return
	}
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

	// Connecting is stopped.
	if stopped {
		// If nobody else connected, cancel automatically.
		if peers.Len() == 1 {
			onExit(connCancelled)
			return
		}
		// The user either confirms that all
		// connected devices are good or cancels.
		if !newBtns.S && oldBtns.S {
			if dialogRight {
				onExit(connFinished)
			} else {
				onExit(connCancelled)
			}
			return
		}
	}
}

func onExit(status uint32) {
	exited = true
	firefly.DrawImage(splash, firefly.Point{})
	setConnStatus(status)
}

func handlePad() {
	if !stopped {
		return
	}
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
	if exited {
		return
	}
	drawBackgroundGrid()
	drawBackgroundBox()
	if !stopped {
		drawLoadingBar()
	}
	drawPeers()
	drawButtons()
}

func drawBackgroundGrid() {
	const cellSize = 10
	firefly.ClearScreen(firefly.ColorWhite)
	lineStyle := firefly.L(firefly.ColorLightGray, 1)
	for x := cellSize; x < firefly.Width; x += cellSize {
		firefly.DrawLine(
			firefly.P(x, 0),
			firefly.P(x, firefly.Height),
			lineStyle,
		)
	}
	for y := cellSize; y < firefly.Height; y += cellSize {
		firefly.DrawLine(
			firefly.P(0, y),
			firefly.P(firefly.Width, y),
			lineStyle,
		)
	}
}

func drawBackgroundBox() {
	const margin = 15
	size := firefly.S(firefly.Width-margin*2, firefly.Height-margin*2)
	firefly.DrawRoundedRect(
		firefly.P(margin+1, margin+1),
		size,
		firefly.S(4, 4),
		firefly.Solid(firefly.ColorBlack),
	)
	firefly.DrawRoundedRect(
		firefly.P(margin, margin),
		size,
		firefly.S(4, 4),
		firefly.Style{
			FillColor:   firefly.ColorWhite,
			StrokeColor: firefly.ColorBlack,
			StrokeWidth: 1,
		},
	)
}

func drawLoadingBar() {
	point := firefly.Point{X: X, Y: Y - FontHeight}
	text := "scanning..."
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
			firefly.DrawText("you:", font, point, firefly.ColorGreen)
			point.X += FontWidth * 5
		}
		firefly.DrawText(name, font, point, firefly.ColorBlack)
		i++
	}
}

func drawButtons() {
	// Draw "stop"/"cancel" button depending on
	// if there are any other peers connected.
	if !stopped {
		text := " stop"
		if peers.Len() == 1 {
			text = "cancel"
		}
		x := (firefly.Width - btnWidth) / 2
		drawButton(x, text, !dialogRight)
	}

	// Draw "cancel" button
	if stopped {
		x := firefly.Width/2 - (btnWidth + btnWidth/2)
		drawButton(x, "cancel", !dialogRight)
	}

	// Draw "ok" button.
	if stopped {
		x := firefly.Width/2 + btnWidth/2
		drawButton(x, "  ok", dialogRight)
	}

}

func drawButton(x int, t string, selected bool) {
	y := 120
	if selected {
		corner := firefly.Size{W: 4, H: 4}
		firefly.DrawRoundedRect(
			firefly.Point{X: x + 1, Y: y + 1},
			firefly.Size{W: btnWidth, H: 12},
			corner,
			firefly.Style{
				FillColor: firefly.ColorBlack,
			},
		)
		firefly.DrawRoundedRect(
			firefly.Point{X: x, Y: y},
			firefly.Size{W: btnWidth, H: 12},
			corner,
			firefly.Style{
				FillColor:   firefly.ColorLightGreen,
				StrokeColor: firefly.ColorBlack,
				StrokeWidth: 1,
			},
		)
	}

	point := firefly.Point{X: x + 3, Y: y + 7}
	firefly.DrawText(t, font, point, firefly.ColorBlack)
}
