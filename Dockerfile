FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY frontend_flask.py .
COPY templates/ ./templates/
COPY app/static/ ./app/static/
EXPOSE 80
CMD ["python", "frontend_flask.py"]
