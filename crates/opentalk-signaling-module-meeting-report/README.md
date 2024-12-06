# OpenTalk Meeting-Report Signaling Module

This module can be used to export reports that provide information about a
meeting. A typical report would be the participant list which contains general
information such as the meeting name, start and end date, as well as a list of
participants present at the time of report creation. These reports are typically
rendered as PDF files.

The tests contain a few sets of example data as safeguards against unintentional
changes in the report format. The tests internally perform this sequence:

- Generate the PDF export in memory using one of the sample datasets
- *Optionally* export the PDF into a file (see below)
- Read the text from the generated PDF
- Verify the text contents against snapshot data with [`insta`](https://docs.rs/insta/)

In order to export the PDF as a file for visual checks, as well as the
corresponding JSON data that is used for generating the files, set the
`OPENTALK_REPORT_DUMP_PATH` environment file to a path. This will cause the
export to happen into that directory. If the directory does not exist yet, it
will be created. This can be used to dump the report either during a test run or
when running the OpenTalk controller service.
