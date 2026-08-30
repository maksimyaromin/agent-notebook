/* The map's behaviour: how the graph is arranged, which names it can show
   without smearing them over each other, and what a tile opens when a
   reader picks one.

   Nothing here writes to a notebook. What a reader decides on this page
   they carry back to the CLI, which stays the single write path. */

(function () {
  "use strict";

  var data = JSON.parse(document.getElementById("anb-data").textContent);
  var stage = document.getElementById("stage");
  var viewport = document.getElementById("viewport");
  var el = function (id) { return document.getElementById(id); };

  /* ---- what the page is made of ---------------------------------------- */

  var tiles = [];
  var tileOf = new Map();
  Array.prototype.forEach.call(document.querySelectorAll("#nodes > g"), function (group) {
    var tile = {
      id: group.dataset.id,
      group: group,
      label: group.querySelector("text"),
      radius: Number(group.querySelector(".tile").getAttribute("r")),
      reach: group.querySelector(".reach"),
      degree: Number(group.dataset.degree),
      state: group.dataset.state,
      archived: group.classList.contains("archived"),
      record: null,
      x: 0,
      y: 0,
      dx: 0,
      dy: 0,
      shown: true
    };
    tiles.push(tile);
    tileOf.set(tile.id, tile);
  });
  data.nodes.forEach(function (record) { tileOf.get(record.id).record = record; });

  var lines = Array.prototype.map.call(document.querySelectorAll("#edges > polyline"), function (line) {
    return {
      line: line,
      kind: line.dataset.kind,
      from: tileOf.get(line.dataset.source),
      to: tileOf.get(line.dataset.target),
      points: [],
      shown: true
    };
  });

  var neighbours = new Map(tiles.map(function (tile) { return [tile.id, new Set([tile.id])]; }));
  lines.forEach(function (edge) {
    neighbours.get(edge.from.id).add(edge.to.id);
    neighbours.get(edge.to.id).add(edge.from.id);
  });

  /* ---- the space a name needs ------------------------------------------ */

  // The gap between a tile and the name under it, and how far the name's
  // own box reaches past its glyphs before it counts as touching another.
  var LABEL_GAP = 6;
  var LABEL_CLEARANCE = 3;

  // The smallest a name may render at before it is a smudge rather than a
  // word. A name shrinking with the map stops here and holds its size, so
  // what a reader can see they can also read; the ones that no longer fit
  // fade out, and zooming in brings them back.
  var LEGIBLE = 7;

  // Measured, never counted: an id's width in the font it is actually drawn
  // in is what has to be reserved, and a width guessed from characters
  // under-reserves exactly where ids run longest.
  tiles.forEach(function (tile) {
    tile.labelWidth = tile.label.getComputedTextLength();
    tile.labelSize = parseFloat(getComputedStyle(tile.label).fontSize) || 9;
    tile.label.setAttribute("y", tile.radius + LABEL_GAP);
  });

  var boxWidth = function (tile) { return Math.max(tile.radius * 2, tile.labelWidth); };
  var boxHeight = function (tile) { return tile.radius * 2 + LABEL_GAP + tile.labelSize; };

  var onTheMap = function () { return tiles.filter(function (tile) { return tile.shown; }); };
  var drawnLines = function () { return lines.filter(function (edge) { return edge.shown; }); };

  /* ---- the two arrangements -------------------------------------------- */

  // A reader picks tiles here and carries a batch of instructions back to
  // the CLI, so the arrangement the map opens in has to hold a whole slice
  // in one viewport. `layered` ranks a dependency chain top to bottom and
  // grows as wide as the notebook is broad.
  //
  // It rides in the address, so a map handed on opens the way it was left.
  var arrangement = location.hash === "#layered" ? "layered" : "web";

  var layered = function () {
    // A multigraph, because a Task both waiting on and born from the same
    // record draws two lines, and a plain graph holds one edge per pair.
    var graph = new dagre.graphlib.Graph({ directed: true, multigraph: true });
    graph.setGraph({ rankdir: "TB", nodesep: 26, ranksep: 70, marginx: 60, marginy: 60 });
    graph.setDefaultEdgeLabel(function () { return {}; });
    onTheMap().forEach(function (tile) {
      graph.setNode(tile.id, { width: boxWidth(tile), height: boxHeight(tile) });
    });
    // Named by kind: a Task both waiting on and born from the same record
    // draws two lines, and one graphlib edge would hold only one of them.
    drawnLines().forEach(function (edge) { graph.setEdge(edge.from.id, edge.to.id, {}, edge.kind); });
    dagre.layout(graph);

    onTheMap().forEach(function (tile) {
      var box = graph.node(tile.id);
      // The name hangs under the tile inside the box the layout reserved,
      // so the tile sits at the box's top rather than at its middle.
      tile.x = box.x;
      tile.y = box.y - boxHeight(tile) / 2 + tile.radius;
    });
    drawnLines().forEach(function (edge) {
      var laid = graph.edge(edge.from.id, edge.to.id, edge.kind);
      edge.points = laid && laid.points ? laid.points.map(function (at) { return { x: at.x, y: at.y }; }) : [];
    });
  };

  // The web: the same slice folded into one viewport. It starts from the
  // notebook's own order and runs a fixed number of passes, so two maps of
  // an unchanged notebook are the same map.
  var GOLDEN_ANGLE = 2.39996323;
  var PASSES = 400;
  var IDEAL_EDGE = 130;
  // How many clearances away one tile still pushes another. Beyond it the
  // web keeps growing with every unrelated Task the notebook holds, until
  // a whole slice no longer fits one viewport.
  var REACH = 3;

  var web = function () {
    var nodes = onTheMap();
    var edges = drawnLines();
    var span = Math.max(400, Math.sqrt(nodes.length) * 220);
    nodes.forEach(function (tile, index) {
      var reach = (span / 2) * Math.sqrt((index + 0.5) / nodes.length);
      tile.x = Math.cos(index * GOLDEN_ANGLE) * reach;
      tile.y = Math.sin(index * GOLDEN_ANGLE) * reach;
    });

    for (var pass = 0; pass < PASSES; pass++) {
      var cooling = 1 - pass / PASSES;
      nodes.forEach(function (tile) { tile.dx = 0; tile.dy = 0; });
      for (var i = 0; i < nodes.length; i++) {
        for (var j = i + 1; j < nodes.length; j++) {
          apart(nodes[i], nodes[j]);
        }
      }
      edges.forEach(together);
      nodes.forEach(function (tile) {
        tile.x += clamp(tile.dx, 30) * cooling - tile.x * 0.01;
        tile.y += clamp(tile.dy, 30) * cooling - tile.y * 0.01;
      });
    }
    edges.forEach(function (edge) { edge.points = []; });
  };

  // How far apart two tiles have to sit for their names to clear each
  // other: the same measurement the layered ranks separate by.
  var room = function (one, other) {
    return (boxWidth(one) + boxWidth(other)) / 2 + LABEL_CLEARANCE * 2;
  };

  // Two tiles push each other apart until their names clear, and no
  // further. An unbounded push drives a Task nothing links to off the map
  // by every Task it has nothing to do with, and the reader is left
  // panning to find it.
  var apart = function (one, other) {
    var dx = one.x - other.x;
    var dy = one.y - other.y;
    var far = Math.max(1, Math.hypot(dx, dy));
    var clear = room(one, other);
    if (far > clear * REACH) { return; }
    var push = (clear * clear) / far / 4;
    one.dx += (dx / far) * push;
    one.dy += (dy / far) * push;
    other.dx -= (dx / far) * push;
    other.dy -= (dy / far) * push;
  };

  // A line pulls its two ends together, never closer than the room their
  // names need: a shorter edge would only be undone by the push above.
  var together = function (edge) {
    var dx = edge.to.x - edge.from.x;
    var dy = edge.to.y - edge.from.y;
    var far = Math.max(1, Math.hypot(dx, dy));
    var rest = Math.max(IDEAL_EDGE, room(edge.from, edge.to));
    var pull = ((far - rest) / far) * 0.25;
    edge.from.dx += dx * pull;
    edge.from.dy += dy * pull;
    edge.to.dx -= dx * pull;
    edge.to.dy -= dy * pull;
  };

  var clamp = function (value, limit) { return Math.max(-limit, Math.min(limit, value)); };

  var arrange = function (name) {
    arrangement = name;
    if (location.hash !== "#" + name) { location.hash = name; }
    Array.prototype.forEach.call(document.querySelectorAll("[data-layout]"), function (button) {
      button.classList.toggle("on", button.dataset.layout === name);
    });
    if (name === "layered") { layered(); } else { web(); }
    draw();
    fit();
    // Named on the spot rather than on the next frame: a map that opened
    // with every name printed over its neighbour and sorted itself out a
    // frame later is a map whose first impression is the smear.
    relabel();
  };

  /* ---- putting it on the page ------------------------------------------ */

  var draw = function () {
    tiles.forEach(function (tile) {
      tile.group.setAttribute("transform", "translate(" + round(tile.x) + " " + round(tile.y) + ")");
    });
    lines.forEach(function (edge) {
      edge.line.setAttribute("points", route(edge).map(function (at) {
        return round(at.x) + "," + round(at.y);
      }).join(" "));
    });
  };

  var round = function (value) { return Math.round(value * 100) / 100; };

  // The layout's own points, with each end pulled back onto the circle it
  // meets: the arrowhead lands on the tile a reader sees, not on the box
  // the layout reserved around its name.
  var route = function (edge) {
    var through = [{ x: edge.from.x, y: edge.from.y }]
      .concat(edge.points.slice(1, Math.max(1, edge.points.length - 1)))
      .concat([{ x: edge.to.x, y: edge.to.y }]);
    var head = towards(through[0], through[1], edge.from.radius);
    var tail = towards(through[through.length - 1], through[through.length - 2], edge.to.radius + 4);
    through[0] = head;
    through[through.length - 1] = tail;
    return through;
  };

  var towards = function (from, to, distance) {
    var dx = to.x - from.x;
    var dy = to.y - from.y;
    var far = Math.max(0.001, Math.hypot(dx, dy));
    return { x: from.x + (dx / far) * distance, y: from.y + (dy / far) * distance };
  };

  /* ---- panning and zooming --------------------------------------------- */

  var view = { x: 0, y: 0, k: 1 };

  // What a pointer has to land in to mean a tile, held at the same size on
  // screen however far the map is zoomed out — a reach that shrank with the
  // map would put a leaf back out of reach exactly when the map is at its
  // most crowded.
  var REACH = 13;

  var applyView = function () {
    viewport.setAttribute("transform", "translate(" + round(view.x) + " " + round(view.y) + ") scale(" + round(view.k) + ")");
    tiles.forEach(function (tile) {
      tile.reach.setAttribute("r", round(Math.max(tile.radius, REACH / view.k)));
    });
    scheduleLabels();
  };

  // The ground a map is fitted into: the viewport less the panels laid over
  // it. Fitting to the whole viewport slides names under the controls, and
  // a name a reader cannot read is a name the map did not show.
  var clearGround = function () {
    var chrome = el("chrome").getBoundingClientRect();
    var legend = el("legend").getBoundingClientRect();
    var record = el("record");
    var left = chrome.right + PANEL_GAP;
    var right = record.hidden
      ? stage.clientWidth - PANEL_GAP
      : record.getBoundingClientRect().left - PANEL_GAP;
    return {
      left: left,
      top: PANEL_GAP,
      right: Math.max(left + MIN_GROUND, right),
      bottom: Math.max(PANEL_GAP + MIN_GROUND, legend.top - PANEL_GAP)
    };
  };

  var PANEL_GAP = 20;
  var MIN_GROUND = 200;

  // What a name costs on screen at a given zoom. Above the legibility floor
  // a name shrinks with the map and its cost falls; at the floor it holds a
  // fixed number of pixels however far the map is zoomed out. Fitting on the
  // unfloored size reserves less than the map draws, and the outermost names
  // land past the edge.
  var labelPixels = function (tile, k) {
    return tile.labelWidth * Math.max(tile.labelSize * k, LEGIBLE) / tile.labelSize;
  };

  // The screen the drawing needs at a given zoom. Positions scale with the
  // zoom, floored names do not, so the two are measured together here rather
  // than scaled from one box afterwards.
  var spanAt = function (shown, k) {
    var left = Infinity, right = -Infinity, top = Infinity, bottom = -Infinity;
    shown.forEach(function (tile) {
      var half = Math.max(tile.radius * k, labelPixels(tile, k) / 2);
      left = Math.min(left, tile.x * k - half);
      right = Math.max(right, tile.x * k + half);
      top = Math.min(top, (tile.y - tile.radius) * k);
      bottom = Math.max(bottom, (tile.y + tile.radius + LABEL_GAP) * k + Math.max(tile.labelSize * k, LEGIBLE));
    });
    return { left: left, right: right, top: top, bottom: bottom };
  };

  var fit = function () {
    var ground = clearGround();
    var midX = (ground.left + ground.right) / 2;
    var midY = (ground.top + ground.bottom) / 2;
    var shown = onTheMap();
    if (!shown.length) { view = { x: midX, y: midY, k: 1 }; applyView(); return; }

    // The span grows with the zoom and never shrinks, so the largest zoom
    // that still fits is found by halving the interval rather than by
    // dividing one span once — one division answers for the size the names
    // would have had at a zoom that was not chosen.
    var wide = ground.right - ground.left;
    var tall = ground.bottom - ground.top;
    var low = 0.05, high = 2;
    for (var step = 0; step < 24; step++) {
      var k = (low + high) / 2;
      var span = spanAt(shown, k);
      if (span.right - span.left <= wide && span.bottom - span.top <= tall) { low = k; } else { high = k; }
    }
    view.k = low;
    var at = spanAt(shown, low);
    view.x = midX - (at.left + at.right) / 2;
    view.y = midY - (at.top + at.bottom) / 2;
    applyView();
  };

  stage.addEventListener("wheel", function (event) {
    event.preventDefault();
    var factor = Math.exp(-event.deltaY * 0.0015);
    var next = Math.max(0.05, Math.min(6, view.k * factor));
    view.x = event.clientX - ((event.clientX - view.x) * next) / view.k;
    view.y = event.clientY - ((event.clientY - view.y) * next) / view.k;
    view.k = next;
    applyView();
  }, { passive: false });

  var dragging = null;
  // Whether the gesture that just ended moved the map. A pan releases the
  // pointer before the click arrives, and a click is what opens and closes
  // a record, so panning would otherwise shut the record a reader is
  // holding open.
  var panned = false;
  var TREMOR = 4;

  stage.addEventListener("pointerdown", function (event) {
    dragging = { x: event.clientX - view.x, y: event.clientY - view.y, from: [event.clientX, event.clientY] };
    panned = false;
    stage.classList.add("panning");
    stage.setPointerCapture(event.pointerId);
  });
  stage.addEventListener("pointermove", function (event) {
    if (!dragging) { return; }
    // A hand shakes. Only a move past the tremor is a pan; anything
    // smaller stays the click a reader meant it to be.
    if (Math.hypot(event.clientX - dragging.from[0], event.clientY - dragging.from[1]) > TREMOR) {
      panned = true;
    }
    view.x = event.clientX - dragging.x;
    view.y = event.clientY - dragging.y;
    applyView();
  });
  var endDrag = function () { dragging = null; stage.classList.remove("panning"); };
  stage.addEventListener("pointerup", endDrag);
  stage.addEventListener("pointercancel", endDrag);

  /* ---- the names, and the smear they must not become -------------------- */

  // The tile whose record is open, and the one under the pointer: both are
  // named whatever the placement below decides, since a reader looking at
  // one tile is asking for exactly its name.
  var picked = null;
  var hovered = null;

  var mustName = function () {
    var named = new Set();
    if (picked) { neighbours.get(picked.id).forEach(function (id) { named.add(id); }); }
    if (hovered) { named.add(hovered.id); }
    return named;
  };

  // The size a name is drawn at: its own, until the map is zoomed far
  // enough out that its own would be unreadable, and the floor from there
  // on. The floor is a rendered size, so it is the same word on screen
  // whatever the zoom, and what stops fitting is dropped rather than shrunk
  // into a smudge.
  var drawnSize = function (tile) { return Math.max(tile.labelSize, LEGIBLE / view.k); };

  // Where something actually lands, in the pixels a reader sees, with the
  // clearance a neighbour has to keep off it. Asked of the browser rather
  // than modelled here: a model of where glyphs fall is a second answer to
  // a question the browser has already settled, and the two drift.
  var groundOf = function (element) {
    var box = element.getBoundingClientRect();
    return {
      x: box.left - LABEL_CLEARANCE,
      y: box.top - LABEL_CLEARANCE,
      w: box.width + LABEL_CLEARANCE * 2,
      h: box.height + LABEL_CLEARANCE * 2
    };
  };

  var overlaps = function (one, other) {
    return one.x < other.x + other.w && other.x < one.x + one.w &&
      one.y < other.y + other.h && other.y < one.y + one.h;
  };

  // Names are placed in the order a reader would miss them: the tile they
  // are reading and its neighbourhood, then the one under the pointer, then
  // the tiles the rest of the slice leans on. A name landing on ground a
  // name already took is dropped rather than drawn over it.
  //
  // Every size is written before any ground is asked for, so the browser
  // lays the names out once for the whole pass rather than once per name.
  var relabel = function () {
    var named = mustName();
    var shown = onTheMap();
    shown.forEach(function (tile) {
      var size = drawnSize(tile);
      // Inline, because the stylesheet gives a hub its own size and a
      // presentation attribute would lose to it.
      tile.label.style.fontSize = round(size) + "px";
      tile.label.setAttribute("y", round(tile.radius + LABEL_GAP));
    });

    var ground = new Map(shown.map(function (tile) { return [tile.id, groundOf(tile.label)]; }));
    var taken = [];
    shown.slice().sort(function (one, other) {
      var byDuty = Number(named.has(other.id)) - Number(named.has(one.id));
      if (byDuty) { return byDuty; }
      if (other.degree !== one.degree) { return other.degree - one.degree; }
      return one.id < other.id ? -1 : 1;
    }).forEach(function (tile) {
      var box = ground.get(tile.id);
      if (!named.has(tile.id) && taken.some(function (held) { return overlaps(held, box); })) {
        tile.label.classList.add("faded");
        tile.label.removeAttribute("data-box");
        return;
      }
      taken.push(box);
      tile.label.classList.remove("faded");
      // The ground this name took, as the browser laid it out: what a
      // reader annotates, and what a check reads back to prove no two names
      // share a patch of screen.
      tile.label.setAttribute("data-box", [round(box.x), round(box.y), round(box.w), round(box.h)].join(","));
    });
    tiles.filter(function (tile) { return !tile.shown; }).forEach(function (tile) {
      tile.label.removeAttribute("data-box");
    });
  };

  var pending = null;
  var scheduleLabels = function () {
    if (pending) { return; }
    pending = requestAnimationFrame(function () { pending = null; relabel(); });
  };

  /* ---- what is on the map ---------------------------------------------- */

  var hiddenStates = new Set();
  var showArchived = true;
  var showBorn = true;

  var settleVisibility = function () {
    tiles.forEach(function (tile) {
      tile.shown = !hiddenStates.has(tile.state) && (showArchived || !tile.archived);
      tile.group.setAttribute("visibility", tile.shown ? "visible" : "hidden");
    });
    lines.forEach(function (edge) {
      edge.shown = edge.from.shown && edge.to.shown && (showBorn || edge.kind !== "born");
      edge.line.setAttribute("visibility", edge.shown ? "visible" : "hidden");
    });
    el("tally").textContent = onTheMap().length + " of " + tiles.length;
  };

  var restage = function () {
    settleVisibility();
    arrange(arrangement);
  };

  // One chip per state actually on this map: a legend naming states the
  // slice does not hold would be a legend about another notebook.
  var states = [];
  tiles.forEach(function (tile) {
    if (states.indexOf(tile.state) === -1) { states.push(tile.state); }
  });
  states.sort().forEach(function (state) {
    var chip = document.createElement("span");
    chip.className = "chip on";
    chip.dataset.state = state;
    chip.appendChild(document.createElement("i"));
    chip.appendChild(document.createTextNode(state));
    chip.addEventListener("click", function () {
      if (hiddenStates.has(state)) { hiddenStates.delete(state); } else { hiddenStates.add(state); }
      chip.classList.toggle("on", !hiddenStates.has(state));
      restage();
    });
    el("states").appendChild(chip);
  });

  el("show-archived").addEventListener("change", function (event) {
    showArchived = event.target.checked;
    restage();
  });
  el("show-born").addEventListener("change", function (event) {
    showBorn = event.target.checked;
    restage();
  });

  /* ---- the neighbourhood under the pointer ------------------------------ */

  var lift = function (tile) {
    var near = neighbours.get(tile.id);
    tiles.forEach(function (other) { other.group.classList.toggle("dim", !near.has(other.id)); });
    tiles.forEach(function (other) { other.group.classList.toggle("near", near.has(other.id) && other !== tile); });
    lines.forEach(function (edge) {
      var touching = edge.from === tile || edge.to === tile;
      edge.line.classList.toggle("dim", !touching);
      edge.line.classList.toggle("near", touching);
    });
  };

  var settle = function () {
    tiles.forEach(function (tile) { tile.group.classList.remove("dim", "near"); });
    lines.forEach(function (edge) { edge.line.classList.remove("dim", "near"); });
  };

  tiles.forEach(function (tile) {
    tile.group.addEventListener("mouseenter", function () {
      hovered = tile;
      if (!picked) { lift(tile); }
      scheduleLabels();
    });
    tile.group.addEventListener("mouseleave", function () {
      hovered = null;
      if (!picked) { settle(); }
      scheduleLabels();
    });
    tile.group.addEventListener("click", function (event) {
      event.stopPropagation();
      if (panned) { return; }
      pick(tile);
    });
  });

  /* ---- the record a tile stands for ------------------------------------- */

  var text = function (tag, value, className) {
    var node = document.createElement(tag);
    node.textContent = value;
    if (className) { node.className = className; }
    return node;
  };

  var idRow = function (ids) {
    var wrap = document.createElement("div");
    wrap.className = "ids";
    ids.forEach(function (id) {
      var button = text("button", id);
      var target = tileOf.get(id);
      if (!target) {
        // Only Tasks get tiles, and a slice holds only part of them, so a
        // record cites ids this map cannot travel to. They are still what
        // the record says.
        button.disabled = true;
      } else {
        button.addEventListener("click", function () { pick(target); centre(target); });
      }
      wrap.appendChild(button);
    });
    return wrap;
  };

  var show = function (record) {
    var body = el("record-body");
    body.replaceChildren();
    body.appendChild(text("h2", record.id));
    if (record.title) { body.appendChild(text("p", record.title, "lede")); }

    var list = document.createElement("dl");
    record.fields.forEach(function (pair) {
      list.appendChild(text("dt", pair[0]));
      list.appendChild(text("dd", pair[1]));
    });
    body.appendChild(list);

    if (record.body.trim()) { body.appendChild(text("pre", record.body)); }

    if (record.mentions.length) {
      body.appendChild(text("h3", "mentions"));
      body.appendChild(idRow(record.mentions));
    }
    el("record").hidden = false;
  };

  var pick = function (tile) {
    if (picked) { picked.group.classList.remove("picked"); }
    picked = tile;
    tile.group.classList.add("picked");
    lift(tile);
    show(tile.record);
    scheduleLabels();
  };

  var release = function () {
    if (picked) { picked.group.classList.remove("picked"); }
    picked = null;
    settle();
    el("record").hidden = true;
    scheduleLabels();
  };

  var centre = function (tile) {
    view.x = stage.clientWidth / 2 - tile.x * view.k;
    view.y = stage.clientHeight / 2 - tile.y * view.k;
    applyView();
  };

  stage.addEventListener("click", function () {
    if (panned) { return; }
    release();
  });
  el("close").addEventListener("click", release);
  document.addEventListener("keydown", function (event) { if (event.key === "Escape") { release(); } });

  /* ---- finding one tile among many -------------------------------------- */

  var matches = function (record, needle) {
    return (record.id + " " + (record.title || "") + " " + record.body).toLowerCase().indexOf(needle) !== -1;
  };

  el("search").addEventListener("input", function (event) {
    var needle = event.target.value.trim().toLowerCase();
    tiles.forEach(function (tile) {
      tile.group.classList.toggle("hit", needle !== "" && matches(tile.record, needle));
    });
    scheduleLabels();
  });

  /* ---- open -------------------------------------------------------------- */

  Array.prototype.forEach.call(document.querySelectorAll("[data-layout]"), function (button) {
    button.addEventListener("click", function () { arrange(button.dataset.layout); });
  });
  el("fit").addEventListener("click", fit);
  window.addEventListener("resize", fit);
  // The address is the arrangement, so editing it in place changes the map
  // rather than leaving a link that only works on a fresh open.
  window.addEventListener("hashchange", function () {
    var asked = location.hash === "#layered" ? "layered" : "web";
    if (asked !== arrangement) { arrange(asked); }
  });

  settleVisibility();
  arrange(arrangement);
})();
