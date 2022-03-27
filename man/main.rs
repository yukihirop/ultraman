#[cfg(feature = "man")]
extern crate roff;
use roff::*;
use chrono;

fn main() {
    let current = format!("Ultraman {}", env!("CARGO_PKG_VERSION"));
    let current_date = chrono::Utc::now();
    let footer = format!("{}", current_date.format("%B %Y"));
    let page = Roff::new("ultraman", 1, footer.as_str(), current.as_str(), "Ultraman Manual")
    .section("name", &["ultraman - modify files by randomly changing bits"])
    .section("synopsis", &[
      lf(&[bold("ultraman"), " ".into(), bold("start"), " ".into(), "[process]".into()]),
      lf(&[bold("ultraman"), " ".into(), bold("run"),   " ".into(), "<command>".into()]),
      lf(&[bold("ultraman"), " ".into(), bold("export")," ".into(), "<format>".into(), " ".into(), "[location]".into()]),
    ])
    .section("description", &[
        bold("ultraman"), " ".into(),
        "is a manager for Procfile-based applications. Its aim is to abstract away the details of the Procfile format, and allow you to either run your application directly or export it to some other process management format.".into(),
    ])
    .section("start", &[
      p(&["If no additional parameters are passed".into(), ", ".into(), bold("ultraman"), " ".into(), "will run one instance of each type of process defined in your Procfile.".into()]),
      p(&["The following options control how the application is run:"]),
      p(&[
        list(
            &[bold("-m"), ", ".into(), bold("--formation"), " ".into(), "[default: all=1]".into()],
            &["Specify the number of each process type to run. The value passed in should be in the format process=num,process=num"]
        )
      ]),
      p(&[
        list(
            &[bold("-e"), ", ".into(), bold("--env")," ".into(), "[default: .env]".into()],
            &["Specify an environment file to load"]
        )
      ]),
      p(&[
        list(
            &[bold("-f"), ", ".into(), bold("--procfile"), " ".into(), "[default: Procfile]".into()],
            &["Specify an alternate Procfile to load, implies -d at the Procfile root"]
        )
      ]),
      p(&[
        list(
            &[bold("-p"), ", ".into(), bold("--port"), " ".into(), "[default: 5000]".into()],
            &["Specify which port to use as the base for this application. Should be a multiple of 1000"]
        )
      ]),
      p(&[
        list(
            &[bold("-t"), ", ".into(), bold("--timeout"), " ".into(), "[default: 5]".into()],
            &["Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM"]
        )
      ]),
      p(&[
        list(
            &[bold("-n"), ", ".into(), bold("--no-timestamp")],
            &["Include timestamp in output"]
        )
      ])
    ])
    .section("run", &[
      p(&[bold("ultraman"), " ".into(), "start is used to run your application directly from the command line.".into()]),
      p(&["If no additional parameters are passed".into(), ", ".into(), bold("ultraman"), " ".into(), "will run one instance of each type of process defined in your Procfile.".into()]),
      p(&["If a parameter is passed".into(), ", ".into(), bold("ultraman"), " ".into(), "will run one instance of the specified application type.".into()]),
      p(&["The following options control how the application is run:"]),
      p(&[
        list(
            &[bold("-e"), ", ".into(), bold("--env")," ".into(), "[default: .env]".into()],
            &["Specify an environment file to load"]
        )
      ]),
      p(&[
        list(
            &[bold("-f"), ", ".into(), bold("--procfile"), " ".into(), "[default: Procfile]".into()],
            &["Specify an alternate Procfile to load, implies -d at the Procfile root"]
        )
      ]),
    ])
    .section("export", &[
      p(&[bold("ultraman"), " ".into(), "export is used to export your application to another process management format.".into()]),
      p(&["A location to export can be passed as an argument. This argument may be either required or optional depending on the export format."]),
      p(&["The following options control how the application is run:"]),
      p(&[
        list(
            &[bold("-m"), ", ".into(), bold("--formation"), " ".into(), "[default: all=1]".into()],
            &["Specify the number of each process type to run. The value passed in should be in the format process=num,process=num"]
        )
      ]),
      p(&[
        list(
            &[bold("-e"), ", ".into(), bold("--env")," ".into(), "[default: .env]".into()],
            &["Specify an environment file to load"]
        )
      ]),
      p(&[
        list(
            &[bold("-f"), ", ".into(), bold("--procfile"), " ".into(), "[default: Procfile]".into()],
            &["Specify an alternate Procfile to load, implies -d at the Procfile root"]
        )
      ]),
      p(&[
        list(
            &[bold("-p"), ", ".into(), bold("--port"), " ".into(), "[default: 5000]".into()],
            &["Specify which port to use as the base for this application. Should be a multiple of 1000"]
        )
      ]),
      p(&[
        list(
            &[bold("-t"), ", ".into(), bold("--timeout"), " ".into(), "[default: 5]".into()],
            &["Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM"]
        )
      ]),
      p(&[
        list(
            &[bold("-a"), ", ".into(), bold("--app"), " ".into(), "[default: app]".into()],
            &["Use this name rather than the application's root directory name as the name of the application when exporting"]
        )
      ]),
      p(&[
        list(
            &[bold("-l"), ", ".into(), bold("--log"), " ".into(), "[default: /var/log/app]".into()],
            &["Specify the directory to place process logs in"]
        )
      ]),
      p(&[
        list(
            &[bold("-r"), ", ".into(), bold("--run"), " ".into(), "[default: /var/run/app]".into()],
            &["Specify the pid file directory"]
        )
      ]),
      p(&[
        list(
            &[bold("-T"), ", ".into(), bold("--template")],
            &["Specify an template to use for creating export files"]
        )
      ]),
      p(&[
        list(
            &[bold("-u"), ", ".into(), bold("--user")],
            &["Specify the user the application should be run as. Defaults to the app name"]
        )
      ]),
      p(&[
        list(
            &[bold("-d"), ", ".into(), bold("--root"), "[default: .]".into()],
            &["Specify an alternate application root. This defaults to the directory containing the Procfile"]
        )
      ]),
    ])
    .section("export format", &[
      p(&[bold("ultraman"), " ".into(), "currently supports the following output formats:".into()]),
      ul(&[
        li(4, &["inittab"]),
        li(4, &["launchd"]),
        li(4, &["runnit"]),
        li(4, &["supervisord"]),
        li(4, &["systemd"]),
        li(4, &["upstart"]),
      ])
    ])
    .section("procfile", &[
      s(&["A Procfile should contain both a name for the process and the command used to run it."]),
      nf(4, &[
        lf(&["web: bundle exec rails s"]),
        lf(&["job: bundle exec rake jobs:work"]),
      ]),
      p(&["A process name may contain letters, numbers and the underscore character."]),
      p(&["The special environment variables $PORT and $PS are available within the Procfile. $PORT is the port selected for that process. $PS is the name of the process for the line."]),
      p(&["The $PORT value starts as the base port as specified by -p, then increments by 100 for each new process line. Multiple instances of the same process are assigned $PORT values that increment by 1."])
    ])
    .section("environment", &[
      s(&["If a .env file exists in the current directory, the default environment will be read from it. This file should contain key/value pairs, separated by =, with one key/value pair per line."]),
      nf(4, &[
        lf(&["FOO=foo"]),
        lf(&["BAZ=bar"]),
      ]),
    ])
    .section("default options", &[
      s(&["If a ".into(), bold(".ultraman"), " file exists in the current directory, default options will read from it. This file should".into()]),
      s(&["be in YAML format with the long option name as keys.Exammple:"]),
      nf(4, &[
        lf(&["formation: alpha=0,bravo=1"]),
        lf(&["port: 15000"])
      ])
    ])
    .section("examples", &[
      s(&["Start one instance of each process type, interleave the output on stdout:"]),
      nf(4, &[
        lf(&["$ ultraman start"])
      ]),
      s(&["Export the application in upstart format:"]),
      nf(4, &[
        lf(&["$ ultraman export upstart /etc/init"])
      ]),
      s(&["Run one process type from the application defined in a specific Procfile:"]),
      nf(4, &[
        lf(&["$ ultraman start alpha -f ~/myapp/Procfile"])
      ]),
      s(&["Start all processes except the one named worker:"]),
      nf(4, &[
        lf(&["$ ultraman start -m all=1,worker=0"])
      ])
    ])
    .section("copyright", &[
      s(&[bold("Ultraman"), " ".into(), "Copyright (C) 2020 yukihirop https://github.com/yukihirop/ultraman".into()])
    ]);

    println!("{}", &page.render())
}
