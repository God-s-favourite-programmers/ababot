FROM python:alpine
COPY requirements.txt .
RUN apk update && apk add gcc musl-dev
RUN pip3 install -r requirements.txt
COPY ./src/*.py ./
COPY ./src/cogs/ ./src/cogs/
CMD ["python3", "main.py"]
