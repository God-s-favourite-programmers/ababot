import re
import json
import requests
import datetime
import pytz
from src.cogs.abakus.event import Event

local_timezone = pytz.timezone("Europe/Oslo")

def get_date() -> datetime.datetime:
    today = datetime.datetime.today()
    formatted = today.strftime("%Y-%m-%d")
    return formatted


def list_events() -> list[int]:
    url = f"https://lego.abakus.no/api/v1/events/?date_after={get_date()}"
    data = requests.get(url).json()
    id_list = []
    for event in data["results"]:
        id_list.append(int(event["id"]))
    return id_list


def get_event(eventId:int) -> Event:
    url = "https://lego.abakus.no/api/v1/events/"+str(eventId)
    r = requests.get(url)
    data = json.loads(r.text)
    name = data["title"]
    description = data["description"]
    event_time = data["startTime"]
    thumbnail = data["cover"]
    regex = re.search(
        r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", event_time)
    event_time = datetime.datetime(int(regex.group(1)), int(regex.group(2)), int(
        regex.group(3)), int(regex.group(4)), int(regex.group(5)))
    event_time = pytz.utc.localize(event_time, is_dst=None).astimezone(local_timezone)
    event_location = data["location"]
    try:
        raw_registration_open = data["pools"][0]["activationDate"]
        regex = re.search(
            r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", raw_registration_open)
        registration_open = datetime.datetime(int(regex.group(1)), int(
            regex.group(2)), int(regex.group(3)), int(regex.group(4)), int(regex.group(5)))
        registration_open = pytz.utc.localize(registration_open, is_dst=None).astimezone(local_timezone)
    except IndexError:
        registration_open = None
    return Event(**{
        "name": name,
        "description": description,
        "event_time": event_time,
        "event_location": event_location,
        "registration_open": registration_open,
        "url": "https://abakus.no/events/"+str(eventId),
        "thumbnail": thumbnail
    }) if registration_open != None else None
