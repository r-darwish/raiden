raiden: .build

.build:
	cargo build

test: .env raiden .test
	.env/bin/slash run testing/tests.py

.test:

.env:
	virtualenv .env
	.env/bin/pip install -r testing/requirements.txt
