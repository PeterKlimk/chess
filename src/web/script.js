var prev;

function onDrop (source, target, piece, newPos, oldPos, orientation) {
  $.post("move/" + source + "/" + target, callback = function(data, status) {
    if (data != "valid") {
      board1.position(prev, false);
    }
  });
}

function onDragStart () {
  prev = board1.position();
  console.log(prev);
}

var config = {
  position: 'start',
  draggable: true,
  onDrop: onDrop,
  onDragStart: onDragStart,
}

var board1 = ChessBoard('board1', config);