package main

import "github.com/firefly-zero/firefly-go/firefly"

var (
	peers firefly.Peers
	font  firefly.Font
	frame uint32
	me    firefly.Peer
)

const (
	FontHeight = 10
	FontWidth  = 6
	X          = 120 - 3*13
	Y          = 71
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
}

func render() {
	firefly.ClearScreen(firefly.ColorWhite)
	drawConnecting()
	drawPeers()
}

func drawConnecting() {
	point := firefly.Point{X: X, Y: Y - FontHeight}
	text := "Connecting..."
	firefly.DrawText(text, font, point, firefly.ColorGray)

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
	for i, peer := range peers.Slice() {
		name := firefly.GetName(peer)
		if name == "" {
			name = "???"
		}
		point := firefly.Point{X: X, Y: Y + FontHeight*(i+1)}
		if peer == me {
			firefly.DrawText("you:", font, point, firefly.ColorBlue)
			point.X += FontWidth * 5
		}
		firefly.DrawText(name, font, point, firefly.ColorBlack)
	}
}
