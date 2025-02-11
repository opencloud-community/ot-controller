#set page(
  paper: "a4",
)
#set text(
  size: 10pt,
)

#let data = json("data.json")
#let parse_datetime(s) = toml.decode("date = " + s).date
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]"
#let role_label = (
  moderator: "Moderator",
  user: "User",
  guest: "Guest",
)
#let role_order = (
  moderator: 0,
  user: 1,
  guest: 2,
)
#let visible_kinds = ("user", "guest", "sip")

= Attendance Report

#let metadata_table_content = (
  (
    [Meeting],
    data.title,
  ),
)

#if data.description.len() > 0 {
  metadata_table_content.push((
    [Details],
    data.description
  ))
}

#if "starts_at" in data {
  metadata_table_content.push((
    [Start],
    [ #parse_datetime(data.starts_at).display(datetime_format) ]
  ))
}

#if "ends_at" in data {
  metadata_table_content.push((
    [End],
    [ #parse_datetime(data.ends_at).display(datetime_format) ]
  ))
}

#metadata_table_content.push((
  [Report timezone],
  data.report_timezone
))


#table(
  stroke: none,
  columns: 2,
  ..for (name, content) in metadata_table_content {
    ([*#name*:], [#content])
  }
)

== Participants

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#table(
  stroke: none,
  columns: (auto, auto, 1fr),
  table.header(
    [*Nr*],
    [*Name*],
    [*Role*],
  ),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for (i, participant) in data
    .participants
    .filter(p => "left_at" not in p and visible_kinds.contains(p.kind))
    .sorted(key: p => role_order.at(p.role))
    .enumerate(start: 1) {
    (
      [#i],
      [#participant.name],
      [#role_label.at(participant.role)],
    )
  }
)
