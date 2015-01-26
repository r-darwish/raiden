from contextlib import contextmanager
import os
import random
import shutil
import subprocess
import tempfile
import logbook
from slash import parametrize, fixture


@fixture(name='work_dir')
def work_dir_fixture(this):
    directory = tempfile.mkdtemp()

    def rmdir():
        shutil.rmtree(directory)

    logbook.notice('Working directory: {}', directory)
    this.add_cleanup(rmdir)

    return directory


@fixture(name='source_file')
def source_file_fixture(work_dir):
    path = os.path.join(work_dir, 'source')

    logbook.debug('Creating the source file at {}', path)
    with open(path, 'wb') as f:
        for _ in xrange(1 * 1024):
            f.write(bytes(random.randrange(0, 0xff)))
    logbook.debug('Done creating the source file')

    return path


@parametrize('disks', [3, 5, 10])
def test_sanity(work_dir, source_file, disks):
    cmd = ['target/raiden', 'split', source_file, str(disks)]
    logbook.debug(cmd)
    subprocess.check_call(cmd)

    def merge():
        cmd = ['target/raiden', 'merge', source_file, str(disks)]
        subprocess.check_call(cmd)
        logbook.debug(cmd)

        restored_file = os.path.join(work_dir, 'res__source')
        cmd = ['diff', source_file, restored_file]
        logbook.debug(cmd)
        subprocess.check_call(cmd)

    merge()

    @contextmanager
    def move_aside(disk):
        path = '{}_{}'.format(source_file, disk)
        os.rename(path, path + '~')
        try:
            yield
        finally:
            os.rename(path + '~', path)

    for i in xrange(disks):
        with move_aside(i):
            merge()
