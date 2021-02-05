# Ababot

Discord bot for abakus stuff

[Plans](https://github.com/Areskiko/ababot/projects/1)

> [docker](https://hub.docker.com/repository/docker/areskiko/ababot)

* If you use the Dockerfile add the flags -i and -t, so that you may provide a token manually. If you use a volume you won't have to do this the next time you spin up the container.
* When running it with a volume use ```-v yourvolume:/ababot``` since the bot looks for a token in the /ababot folder.
* ```docker run -d -v ababot-vol:/ababot --name ababot areskiko/ababot:latest```