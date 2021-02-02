FROM python:alpine
COPY requirements.txt .
RUN apk update && apk add gcc musl-dev
RUN pip3 install -r requirements.txt
COPY ./src/*.py ./
COPY ./src/templates/* ./src/templates/
COPY requirements.txt ./
CMD ["python3", "discordBot.py"]
