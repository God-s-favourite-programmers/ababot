FROM python:alpine
COPY requirements.txt .
RUN apk update && apk add gcc musl-dev
RUN pip3 install -r requirements.txt
COPY ./src/*.py ./
COPY ./src/templates/* ./templates/
CMD ["python3", "discordBot.py"]
