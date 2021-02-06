import re
import json
import requests
import datetime


def get_date():
    today = datetime.datetime.today()
    formatted = today.strftime("%Y-%m-%d")
    return formatted


def list_events():
    url = f"https://lego.abakus.no/api/v1/events/?date_after={get_date()}"
    data = requests.get(url).json()
    id_list = []
    for event in data["results"]:
        id_list.append(event["id"])
    return id_list


def get_event(eventId):
    url = "https://lego.abakus.no/api/v1/events/"+str(eventId)
    r = requests.get(url)
    data = json.loads(r.text)
    name = data["title"]
    description = data["description"]
    eventTime = data["startTime"]
    regex = re.search(
        r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", eventTime)
    eventTime = datetime.datetime(int(regex.group(1)), int(regex.group(2)), int(
        regex.group(3)), int(regex.group(4)), int(regex.group(5)))
    eventLocation = data["location"]
    try:
        rawRegistrationOpen = data["pools"][0]["activationDate"]
        regex = re.search(
            r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", rawRegistrationOpen)
        registrationOpen = datetime.datetime(int(regex.group(1)), int(
            regex.group(2)), int(regex.group(3)), int(regex.group(4)), int(regex.group(5)))
    except IndexError:
        registrationOpen = None
    return {
        "name": name,
        "description": description,
        "eventTime": eventTime,
        "eventLocation": eventLocation,
        "registrationOpen": registrationOpen,
        "url": "https://abakus.no/events/"+str(eventId)
    }
