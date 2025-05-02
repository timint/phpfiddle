import http from 'http';
import { spawn } from 'child_process';

export default class PHPServer {
	static start(config, cb) {
		const server = new PHPServer(config);
		return server.run(cb);
	}

	constructor(config) {
		this.php = 'php';
		this.host = '127.0.0.1';
		this.port = 8000;
		this.directory = null;
		this.script = null;
		this.directives = {};
		this.stdio = 'inherit';
		this.env = process.env;

		if (config) {
			Object.keys(config).forEach(name => {
				if (!(name in this)) {
					throw new Error(`Config ${name} is not valid`);
				}

				if (name === 'env') {
					this[name] = Object.assign({}, this[name], config[name]);
				} else {
					this[name] = config[name];
				}
			});
		}
	}

	getParameters() {
		const params = ['-S', `${this.host}:${this.port}`];

		if (this.directory) {
			params.push('-t');
			params.push(this.directory);
		}

		Object.keys(this.directives).forEach(d => {
			params.push('-d');
			params.push(`${d}=${this.directives[d]}`);
		});

		if (this.config) {
			params.push('-c');
			params.push(this.config);
		}

		if (this.script) {
			params.push(this.script);
		}

		return params;
	}

	run(cb) {
		this.process = spawn(this.php, this.getParameters(), {
			stdio: this.stdio,
			env: this.env
		});

		this.process.on('close', () => console.log('PHP Server closed'));
		this.process.on('error', error =>
			console.error('PHP Server error', error)
		);

		if (cb) {
			checkServer(this.host, this.port, () => cb(this));
		} else {
			return new Promise(resolve => {
				checkServer(this.host, this.port, () => resolve(this));
			});
		}
	}

	close() {
		if (!this.process) {
			return;
		}

		this.process.kill();
		delete this.process;
	}

	toString() {
		return `${this.php} ${this.getParameters().join(' ')}`;
	}
}

function checkServer(host, port, cb) {
	function runCheck() {
		http.request({
        method: 'HEAD',
        hostname: host,
        port: port
      }, () => cb()
    )
    .on('error', err => {
      if (err.code === 'ECONNREFUSED') {
        console.error('PHP server not started. Retrying...');
        setTimeout(runCheck, 250);
      } else {
        cb();
      }
    })
    .end();
	}
	runCheck();
}